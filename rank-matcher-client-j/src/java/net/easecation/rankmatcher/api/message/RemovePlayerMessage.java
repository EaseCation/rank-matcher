package net.easecation.rankmatcher.api.message;

import net.easecation.rankmatcher.api.Message;
import net.easecation.rankmatcher.api.MessageType;

public class RemovePlayerMessage implements Message {

    private String arena;
    private String player;

    public static RemovePlayerMessage of(String arena, String player) {
        RemovePlayerMessage message = new RemovePlayerMessage();
        message.arena = arena;
        message.player = player;
        return message;
    }

    @Override
    public MessageType getMessageType() {
        return MessageType.REMOVE_PLAYER;
    }

    @Override
    public String toString() {
        return PROTOCOL_VERSION + "," + getMessageType().getTypeId() + "," + Message.writeString(arena) + "," + Message.writeString(player);
    }

}
