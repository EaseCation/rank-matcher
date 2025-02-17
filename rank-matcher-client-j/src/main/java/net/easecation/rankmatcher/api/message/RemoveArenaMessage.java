package net.easecation.rankmatcher.api.message;

import lombok.Data;
import net.easecation.rankmatcher.api.Message;
import net.easecation.rankmatcher.api.MessageType;

@Data
public class RemoveArenaMessage implements Message {

    private String arena;

    public static RemoveArenaMessage of(String arena) {
        RemoveArenaMessage message = new RemoveArenaMessage();
        message.arena = arena;
        return message;
    }

    @Override
    public MessageType getMessageType() {
        return MessageType.REMOVE_ARENA;
    }

    @Override
    public String toString() {
        return PROTOCOL_VERSION + "," + getMessageType().getTypeId() + "," + Message.writeString(arena);
    }
}
