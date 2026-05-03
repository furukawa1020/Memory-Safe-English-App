# frozen_string_literal: true

require "yaml"

module CatalogOps
  module Renderers
    class YamlCatalogRenderer
      def initialize(records)
        @records = records
      end

      def render
        payload = {
          "contents" => @records.map do |record|
            {
              "id" => record.id,
              "title" => record.title,
              "content_type" => record.content_type,
              "level" => record.level,
              "topic" => record.topic,
              "language" => record.language,
              "raw_text" => record.raw_text,
              "summary_text" => record.summary_text
            }
          end
        }
        YAML.dump(payload)
      end
    end
  end
end
