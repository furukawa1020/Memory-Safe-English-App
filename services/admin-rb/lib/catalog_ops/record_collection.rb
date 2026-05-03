# frozen_string_literal: true

module CatalogOps
  module RecordCollection
    module_function

    def ensure_unique_ids!(records)
      duplicates = records
        .group_by(&:id)
        .select { |_id, items| items.length > 1 }
        .keys
        .sort
      return records if duplicates.empty?

      raise ArgumentError, "duplicate content ids detected: #{duplicates.join(', ')}"
    end
  end
end
