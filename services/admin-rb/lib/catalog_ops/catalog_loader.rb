# frozen_string_literal: true

require "yaml"
require_relative "content_record"

module CatalogOps
  class CatalogLoader
    def self.load(path)
      data = YAML.safe_load_file(path, permitted_classes: [], aliases: false) || {}
      contents = data.fetch("contents") do
        raise ArgumentError, "catalog must include top-level 'contents'"
      end
      raise ArgumentError, "'contents' must be an array" unless contents.is_a?(Array)

      contents.map { |item| ContentRecord.new(item) }
    end
  end
end
