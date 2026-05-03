# frozen_string_literal: true

require_relative "../content_record"

module CatalogOps
  module Importers
    class SqlSeedImporter
      EXPECTED_FIELDS = %w[
        id
        title
        content_type
        level
        topic
        raw_text
        summary_text
        language
      ].freeze

      def initialize(sql_text)
        @sql_text = sql_text
      end

      def import
        values_block = extract_values_block(@sql_text)
        parse_rows(values_block).map do |values|
          raise ArgumentError, "expected #{EXPECTED_FIELDS.length} values, got #{values.length}" unless values.length == EXPECTED_FIELDS.length

          attributes = EXPECTED_FIELDS.zip(values).to_h
          ContentRecord.new(attributes)
        end
      end

      private

      def extract_values_block(sql_text)
        match = sql_text.match(
          /INSERT\s+INTO\s+contents\s*\([^)]+\)\s*VALUES\s*(.+?)(?:ON\s+CONFLICT|;)/im
        )
        raise ArgumentError, "could not find INSERT INTO contents ... VALUES statement" unless match

        match[1]
      end

      def parse_rows(values_block)
        rows = []
        index = 0
        while index < values_block.length
          char = values_block[index]
          if char == "("
            row_values, index = consume_tuple_values(values_block, index + 1)
            rows << row_values
          else
            index += 1
          end
        end
        rows
      end

      def consume_tuple_values(text, start_index)
        values = []
        buffer = +""
        in_string = false
        index = start_index

        while index < text.length
          char = text[index]
          if in_string
            if char == "'"
              if text[index + 1] == "'"
                buffer << "'"
                index += 2
                next
              end
              in_string = false
              index += 1
              next
            end
            buffer << char
            index += 1
            next
          end

          case char
          when "'"
            in_string = true
          when ","
            values << normalize_value(buffer)
            buffer = +""
          when ")"
            values << normalize_value(buffer)
            return [values, index + 1]
          else
            buffer << char
          end
          index += 1
        end
        raise ArgumentError, "unterminated tuple in SQL seed"
      end

      def normalize_value(value)
        value.strip
      end
    end
  end
end
