# frozen_string_literal: true

module CatalogOps
  module Renderers
    class GoSeedRenderer
      def initialize(records, package_name:, function_name:)
        @records = records
        @package_name = package_name
        @function_name = function_name
      end

      def render
        <<~GO
          package #{@package_name}

          import (
          	"time"

          	"memory-safe-english/services/api/internal/domain"
          )

          func #{@function_name}(now time.Time) []domain.Content {
          	return []domain.Content{
          #{render_records}
          	}
          }
        GO
      end

      private

      def render_records
        @records.map do |record|
          <<~RECORD.chomp
            		{
            			ID:          #{go_string(record.id)},
            			Title:       #{go_string(record.title)},
            			ContentType: #{go_string(record.content_type)},
            			Level:       #{go_string(record.level)},
            			Topic:       #{go_string(record.topic)},
            			Language:    #{go_string(record.language)},
            			RawText:     #{go_string(record.raw_text)},
            			SummaryText: #{go_string(record.summary_text)},
            			CreatedAt:   now,
            			UpdatedAt:   now,
            		},
          RECORD
        end.join("\n")
      end

      def go_string(value)
        value.inspect
      end
    end
  end
end
