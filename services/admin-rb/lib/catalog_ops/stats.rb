# frozen_string_literal: true

module CatalogOps
  class Stats
    def initialize(records)
      @records = records
    end

    def to_h
      {
        total: @records.size,
        by_content_type: tally(&:content_type),
        by_level: tally(&:level),
        by_topic: tally(&:topic)
      }
    end

    private

    def tally
      @records.each_with_object(Hash.new(0)) do |record, acc|
        acc[yield(record)] += 1
      end.sort.to_h
    end
  end
end
