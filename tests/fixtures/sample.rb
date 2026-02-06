# A sample Ruby file for testing TODO detection

def main
  # TODO: Add proper error handling
  puts "Hello"

  # HACK(eve): Monkey patch for compatibility
  String.class_eval do
    def blank?
      strip.empty?
    end
  end
end
