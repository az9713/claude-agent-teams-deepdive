// A sample Java file for testing TODO detection

public class Sample {
    // TODO: Add logging framework
    public static void main(String[] args) {
        System.out.println("Hello");
    }

    // BUG(dave, #789): NullPointerException when input is empty
    public void process(String input) {
        input.length();
    }
}
