package net.easecation.rankmatcher.api.message;

import net.easecation.rankmatcher.api.Message;
import net.easecation.rankmatcher.api.MessageType;

public class GetStateMessage implements Message {

    @Override
    public MessageType getMessageType() {
        return MessageType.GET_STATE;
    }

    @Override
    public String toString() {
        return PROTOCOL_VERSION + "," + getMessageType().getTypeId();
    }
}
