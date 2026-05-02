# frozen_string_literal: true

module CatalogOps
  module Renderers
    class SqlSeedRenderer
      def initialize(records)
        @records = records
      end

      def render
        <<~SQL
          INSERT INTO contents (id, title, content_type, level, topic, raw_text, summary_text, language)
          VALUES
          #{values_clause}
          ;
        SQL
      end

      private

      def values_clause
        @records.map do |record|
          "  (#{sql_string(record.id)}, #{sql_string(record.title)}, #{sql_string(record.content_type)}, #{sql_string(record.level)}, #{sql_string(record.topic)}, #{sql_string(record.raw_text)}, #{sql_string(record.summary_text)}, #{sql_string(record.language)})"
        end.join(",\n")
      end

      def sql_string(value)
        "'#{value.gsub("'", "''")}'"
      end
    end
  end
end
