# frozen_string_literal: true

require "minitest/autorun"
require "json"
require "tmpdir"

ROOT = File.expand_path("..", __dir__)
$LOAD_PATH.unshift(File.join(ROOT, "lib"))

require "catalog_ops/catalog_loader"
require "catalog_ops/cli"
require "catalog_ops/stats"
require "catalog_ops/renderers/go_seed_renderer"
require "catalog_ops/renderers/sql_seed_renderer"

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

  def test_cli_can_build_outputs
    Dir.mktmpdir do |dir|
      go_path = File.join(dir, "generated.go")
      sql_path = File.join(dir, "generated.sql")

      CatalogOps::CLI.new(["build-go", SAMPLE_PATH, go_path]).run
      CatalogOps::CLI.new(["build-sql", SAMPLE_PATH, sql_path]).run

      assert File.exist?(go_path)
      assert File.exist?(sql_path)
    end
  end
end
