package net.easecation.rankmatcher.api.message;

import net.easecation.rankmatcher.api.CharReader;
import net.easecation.rankmatcher.api.Message;
import net.easecation.rankmatcher.api.MessageType;

public class FormatErrorMessage implements Message {

    private String message;

    public static FormatErrorMessage of(String message) {
        FormatErrorMessage formatErrorMessage = new FormatErrorMessage();
        formatErrorMessage.message = message;
        return formatErrorMessage;
    }

    @Override
    public void decode(CharReader reader) {
        /*
        let error = self.read_string();
        Ok(Packet::FormatError { error })
         */
        message = reader.readString();
    }

    @Override
    public String toString() {
        return PROTOCOL_VERSION + "," + getMessageType().getTypeId() + "," + Message.writeString(message);
    }

    @Override
    public MessageType getMessageType() {
        return MessageType.FORMAT_ERROR;
    }

}
