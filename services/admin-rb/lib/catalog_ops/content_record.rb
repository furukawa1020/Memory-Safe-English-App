# frozen_string_literal: true

module CatalogOps
  class ContentRecord
    REQUIRED_KEYS = %w[
      id
      title
      content_type
      level
      topic
      language
      raw_text
      summary_text
    ].freeze

    attr_reader :attributes

    def initialize(attributes)
      @attributes = attributes.transform_keys(&:to_s)
      validate!
    end

    def [](key)
      attributes.fetch(key.to_s)
    end

    def id = self["id"]
    def title = self["title"]
    def content_type = self["content_type"]
    def level = self["level"]
    def topic = self["topic"]
    def language = self["language"]
    def raw_text = self["raw_text"]
    def summary_text = self["summary_text"]

    private

    def validate!
      missing = REQUIRED_KEYS.reject do |key|
        value = attributes[key]
        value.is_a?(String) && !value.strip.empty?
      end
      return if missing.empty?

      raise ArgumentError, "missing required keys: #{missing.join(', ')}"
    end
  end
end
