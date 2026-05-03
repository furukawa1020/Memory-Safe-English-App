# frozen_string_literal: true

require "minitest/autorun"
require "json"
require "tmpdir"

ROOT = File.expand_path("..", __dir__)
$LOAD_PATH.unshift(File.join(ROOT, "lib"))

require "catalog_ops/catalog_loader"
require "catalog_ops/cli"
require "catalog_ops/record_collection"
require "catalog_ops/stats"
require "catalog_ops/importers/sql_seed_importer"
require "catalog_ops/renderers/go_seed_renderer"
require "catalog_ops/renderers/sql_seed_renderer"
require "catalog_ops/renderers/yaml_catalog_renderer"

class CatalogOpsTest < Minitest::Test
  SAMPLE_PATH = File.join(ROOT, "data", "sample_content_catalog.yml")

  def test_loader_reads_sample_catalog
    records = CatalogOps::CatalogLoader.load(SAMPLE_PATH)

    assert_equal 3, records.length
    assert_equal "cnt_ruby_001", records.first.id
  end

  def test_stats_counts_by_level
    records = CatalogOps::CatalogLoader.load(SAMPLE_PATH)
    stats = CatalogOps::Stats.new(records).to_h

    assert_equal 3, stats[:total]
    assert_equal 1, stats[:by_level]["intro"]
    assert_equal 1, stats[:by_level]["intermediate"]
    assert_equal 1, stats[:by_level]["upper_intermediate"]
  end

  def test_go_renderer_outputs_domain_content
    records = CatalogOps::CatalogLoader.load(SAMPLE_PATH)
    output = CatalogOps::Renderers::GoSeedRenderer.new(
      records,
      package_name: "memory",
      function_name: "rubySeedCatalog"
    ).render

    assert_includes output, "func rubySeedCatalog"
    assert_includes output, 'ID:          "cnt_ruby_001"'
  end

  def test_sql_renderer_escapes_values
    records = [
      CatalogOps::ContentRecord.new(
        "id" => "cnt_1",
        "title" => "Title",
        "content_type" => "reading",
        "level" => "intro",
        "topic" => "daily",
        "language" => "en",
        "raw_text" => "It's okay.",
        "summary_text" => "Summary"
      )
    ]
    output = CatalogOps::Renderers::SqlSeedRenderer.new(records).render

    assert_includes output, "It''s okay."
  end

  def test_sql_importer_reads_seed_rows
    sql = <<~SQL
      INSERT INTO contents (id, title, content_type, level, topic, language, raw_text, summary_text)
      VALUES
        ('cnt_1', 'Title One', 'reading', 'intro', 'daily', 'en', 'It''s okay.', 'Summary One'),
        ('cnt_2', 'Title Two', 'listening', 'upper_intermediate', 'meeting', 'en', 'One detail first.', 'Summary Two');
    SQL

    records = CatalogOps::Importers::SqlSeedImporter.new(sql).import

    assert_equal 2, records.length
    assert_equal "cnt_1", records.first.id
    assert_equal "It's okay.", records.first.raw_text
    assert_equal "en", records.first.language
    assert_equal "Summary One", records.first.summary_text
    assert_equal "meeting", records.last.topic
  end

  def test_yaml_renderer_outputs_contents_root
    records = CatalogOps::CatalogLoader.load(SAMPLE_PATH)
    output = CatalogOps::Renderers::YamlCatalogRenderer.new(records).render

    assert_includes output, "contents:"
    assert_includes output, "cnt_ruby_001"
  end

  def test_record_collection_rejects_duplicate_ids
    duplicate_records = [
      CatalogOps::ContentRecord.new(
        "id" => "cnt_dup",
        "title" => "One",
        "content_type" => "reading",
        "level" => "intro",
        "topic" => "daily",
        "language" => "en",
        "raw_text" => "One",
        "summary_text" => "One"
      ),
      CatalogOps::ContentRecord.new(
        "id" => "cnt_dup",
        "title" => "Two",
        "content_type" => "listening",
        "level" => "intermediate",
        "topic" => "meeting",
        "language" => "en",
        "raw_text" => "Two",
        "summary_text" => "Two"
      )
    ]

    error = assert_raises(ArgumentError) do
      CatalogOps::RecordCollection.ensure_unique_ids!(duplicate_records)
    end

    assert_includes error.message, "duplicate content ids detected"
  end

  def test_cli_can_build_outputs
    Dir.mktmpdir do |dir|
      go_path = File.join(dir, "generated.go")
      sql_path = File.join(dir, "generated.sql")
      yaml_path = File.join(dir, "imported.yml")

      CatalogOps::CLI.new(["build-go", SAMPLE_PATH, go_path]).run
      CatalogOps::CLI.new(["build-sql", SAMPLE_PATH, sql_path]).run
      CatalogOps::CLI.new(["import-sql", sql_path, yaml_path]).run

      assert File.exist?(go_path)
      assert File.exist?(sql_path)
      assert File.exist?(yaml_path)
    end
  end

  def test_cli_can_build_all_bundle
    Dir.mktmpdir do |dir|
      output_dir = File.join(dir, "bundle")

      CatalogOps::CLI.new(
        [
          "build-all",
          SAMPLE_PATH,
          output_dir,
          "--package",
          "memory",
          "--function",
          "generatedRubySeedCatalog"
        ]
      ).run

      assert File.exist?(File.join(output_dir, "generated_seed.go"))
      assert File.exist?(File.join(output_dir, "generated_seed.sql"))
      assert File.exist?(File.join(output_dir, "catalog_stats.json"))
    end
  end

  def test_cli_can_merge_sql_directory
    Dir.mktmpdir do |dir|
      seed_dir = File.join(dir, "sql")
      FileUtils.mkdir_p(seed_dir)
      File.write(
        File.join(seed_dir, "001.sql"),
        <<~SQL
          INSERT INTO contents (id, title, content_type, level, topic, raw_text, summary_text, language)
          VALUES
            ('cnt_a', 'A', 'reading', 'intro', 'daily', 'A text', 'A summary', 'en');
        SQL
      )
      File.write(
        File.join(seed_dir, "002.sql"),
        <<~SQL
          INSERT INTO contents (id, title, content_type, level, topic, raw_text, summary_text, language)
          VALUES
            ('cnt_b', 'B', 'listening', 'intermediate', 'meeting', 'B text', 'B summary', 'en');
        SQL
      )
      output_path = File.join(dir, "merged.yml")

      CatalogOps::CLI.new(["import-sql-dir", seed_dir, output_path]).run

      assert File.exist?(output_path)
      merged_records = CatalogOps::CatalogLoader.load(output_path)
      assert_equal 2, merged_records.length
      assert_equal %w[cnt_a cnt_b], merged_records.map(&:id)
    end
  end
end
