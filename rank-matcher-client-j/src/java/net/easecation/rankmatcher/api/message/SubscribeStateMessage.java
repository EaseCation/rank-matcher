package net.easecation.rankmatcher.api.message;

import net.easecation.rankmatcher.api.Message;
import net.easecation.rankmatcher.api.MessageType;

public class SubscribeStateMessage implements Message {

    @Override
    public MessageType getMessageType() {
        return MessageType.SUBSCRIBE_STATE;
    }

    @Override
    public String toString() {
        return PROTOCOL_VERSION + "," + getMessageType().getTypeId();
    }

}
