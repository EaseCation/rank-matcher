package net.easecation.rankmatcher.api;

import java.util.function.Supplier;

public class CharReader {

    private final byte[] message;
    private int currentPosition;

    public CharReader(String message) {
        this(message.getBytes());
    }

    public CharReader(byte[] message) {
        this.message = message;
        this.currentPosition = 0;
    }

    public Message readPacket() {
        int protocolVersion = readNumber();
        if (protocolVersion == 1) {
            return readV1();
        } else {
            throw new RuntimeException("不支持除了1之外的版本号。");
        }
    }

    public Message readV1() {
        int id = readNumber();
        Supplier<Message> messageSupplier = Message.MESSAGE_SUPPLIERS.get(id);
        if (messageSupplier != null) {
            return messageSupplier.get();
        } else {
            throw new IllegalArgumentException("Unknown message type: " + id);
        }
    }

    public int readNumber() {
        int start = this.currentPosition;
        while (this.currentPosition < this.message.length && (byte)(this.message[this.currentPosition]) >= '0' && (byte)(this.message[this.currentPosition]) <= '9') {
            this.currentPosition++;
        }
        int result = Integer.parseInt(new String(this.message, start, this.currentPosition - start));
        if (this.currentPosition < this.message.length && (byte)(this.message[this.currentPosition]) == ',') {
            this.currentPosition++;
        }
        return result;
    }

    public String readString() {
        int length = readNumber();
        String result = new String(this.message, currentPosition, length);
        this.currentPosition += length;
        if (this.currentPosition < this.message.length && (byte)(this.message[this.currentPosition]) == ',') {
            this.currentPosition++;
        }
        return result;
    }

}
