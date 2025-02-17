package net.easecation.rankmatcher.api.message;

import lombok.Data;
import net.easecation.rankmatcher.api.Message;
import net.easecation.rankmatcher.api.MessageType;

@Data
public class AddArenaMessage implements Message {

    private String arena;
    private int numPlayers;

    public static AddArenaMessage of(String arena, int numPlayers) {
        AddArenaMessage message = new AddArenaMessage();
        message.arena = arena;
        message.numPlayers = numPlayers;
        return message;
    }

    @Override
    public MessageType getMessageType() {
        return MessageType.ADD_ARENA;
    }

    @Override
    public String toString() {
        return PROTOCOL_VERSION + "," + getMessageType().getTypeId() + "," + Message.writeString(arena) + "," + numPlayers;
    }
}
