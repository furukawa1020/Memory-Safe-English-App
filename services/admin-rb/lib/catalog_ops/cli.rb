# frozen_string_literal: true

require "json"
require "optparse"
require "fileutils"

require_relative "catalog_loader"
require_relative "record_collection"
require_relative "stats"
require_relative "importers/sql_seed_importer"
require_relative "renderers/go_seed_renderer"
require_relative "renderers/sql_seed_renderer"
require_relative "renderers/yaml_catalog_renderer"

module CatalogOps
  class CLI
    def initialize(argv)
      @argv = argv.dup
    end

    def run
      command = @argv.shift
      case command
      when "validate" then validate
      when "stats" then stats
      when "build-go" then build_go
      when "build-sql" then build_sql
      when "import-sql" then import_sql
      when "import-sql-dir" then import_sql_dir
      else
        warn usage
        exit 1
      end
    rescue ArgumentError => e
      warn "error: #{e.message}"
      exit 1
    end

    private

    def validate
      path = fetch_path!
      records = CatalogLoader.load(path)
      puts "ok: #{records.size} records validated"
    end

    def stats
      path = fetch_path!
      records = CatalogLoader.load(path)
      puts JSON.pretty_generate(Stats.new(records).to_h)
    end

    def build_go
      options = {
        package_name: "memory",
        function_name: "generatedRubySeedCatalog"
      }
      parser = OptionParser.new do |opts|
        opts.on("--package NAME") { |value| options[:package_name] = value }
        opts.on("--function NAME") { |value| options[:function_name] = value }
      end
      parser.parse!(@argv)
      input = fetch_path!
      output = fetch_path!
      records = CatalogLoader.load(input)
      RecordCollection.ensure_unique_ids!(records)
      rendered = Renderers::GoSeedRenderer.new(
        records,
        package_name: options[:package_name],
        function_name: options[:function_name]
      ).render
      write_output(output, rendered)
      puts "wrote Go seed: #{output}"
    end

    def build_sql
      input = fetch_path!
      output = fetch_path!
      records = CatalogLoader.load(input)
      RecordCollection.ensure_unique_ids!(records)
      rendered = Renderers::SqlSeedRenderer.new(records).render
      write_output(output, rendered)
      puts "wrote SQL seed: #{output}"
    end

    def import_sql
      input = fetch_path!
      output = fetch_path!
      sql_text = File.read(input)
      records = Importers::SqlSeedImporter.new(sql_text).import
      RecordCollection.ensure_unique_ids!(records)
      rendered = Renderers::YamlCatalogRenderer.new(records).render
      write_output(output, rendered)
      puts "wrote YAML catalog: #{output}"
    end

    def import_sql_dir
      options = {
        pattern: "*.sql"
      }
      parser = OptionParser.new do |opts|
        opts.on("--pattern GLOB") { |value| options[:pattern] = value }
      end
      parser.permute!(@argv)
      input_dir = fetch_path!
      output = fetch_path!
      sql_paths = sql_paths_for_dir(input_dir, options[:pattern])
      raise ArgumentError, "no SQL files matched in #{input_dir}" if sql_paths.empty?

      records = sql_paths.flat_map do |path|
        sql_text = File.read(path)
        next [] unless sql_text.match?(/INSERT\s+INTO\s+contents/i)

        Importers::SqlSeedImporter.new(sql_text).import
      end
      raise ArgumentError, "no importable content seed SQL found in #{input_dir}" if records.empty?
      RecordCollection.ensure_unique_ids!(records)
      rendered = Renderers::YamlCatalogRenderer.new(records).render
      write_output(output, rendered)
      puts "wrote merged YAML catalog: #{output} (#{records.length} records from #{sql_paths.length} files)"
    end

    def fetch_path!
      @argv.shift || raise(ArgumentError, "missing path argument")
    end

    def write_output(path, content)
      FileUtils.mkdir_p(File.dirname(path))
      File.write(path, content)
    end

    def glob_path(input_dir, pattern)
      File.join(File.expand_path(input_dir).tr("\\", "/"), pattern)
    end

    def sql_paths_for_dir(input_dir, pattern)
      expanded = File.expand_path(input_dir)
      Dir.children(expanded)
        .select do |entry|
          File.file?(File.join(expanded, entry)) && File.fnmatch(pattern, entry)
        end
        .sort
        .map { |entry| File.join(expanded, entry) }
    end

    def usage
      <<~USAGE
        usage:
          ruby bin/catalog_ops validate <catalog.yml>
          ruby bin/catalog_ops stats <catalog.yml>
          ruby bin/catalog_ops build-go [--package NAME] [--function NAME] <catalog.yml> <output.go>
          ruby bin/catalog_ops build-sql <catalog.yml> <output.sql>
          ruby bin/catalog_ops import-sql <seed.sql> <output.yml>
          ruby bin/catalog_ops import-sql-dir [--pattern GLOB] <seed_dir> <output.yml>
      USAGE
    end
  end
end
