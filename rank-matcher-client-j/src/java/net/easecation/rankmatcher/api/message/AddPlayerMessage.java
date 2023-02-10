package net.easecation.rankmatcher.api.message;

import lombok.Data;
import net.easecation.rankmatcher.api.Message;
import net.easecation.rankmatcher.api.MessageType;

@Data
public class AddPlayerMessage implements Message {

    private String arena;
    private String player;
    private int rank;

    public static AddPlayerMessage of(String arena, String player, int rank) {
        AddPlayerMessage message = new AddPlayerMessage();
        message.arena = arena;
        message.player = player;
        message.rank = rank;
        return message;
    }

    @Override
    public MessageType getMessageType() {
        return MessageType.ADD_PLAYER;
    }

    @Override
    public String toString() {
        return PROTOCOL_VERSION + "," + getMessageType().getTypeId() + "," + Message.writeString(arena) + "," + Message.writeString(player) + "," + rank;
    }

}
