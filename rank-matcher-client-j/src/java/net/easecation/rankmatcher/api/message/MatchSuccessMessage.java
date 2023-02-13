package net.easecation.rankmatcher.api.message;

import net.easecation.rankmatcher.api.Message;
import net.easecation.rankmatcher.api.MessageType;

public class MatchSuccessMessage implements Message {

    private String arena;
    private String[] players;

    public static MatchSuccessMessage of(String arena, String[] player) {
        MatchSuccessMessage message = new MatchSuccessMessage();
        message.arena = arena;
        message.players = player;
        return message;
    }

    @Override
    public void decode(String[] data) {
        arena = data[3];  // 2 3
        players = new String[Integer.parseInt(data[4])];
        for (int i = 0; i < players.length; i++) {
            players[i] = data[5 + 1 + i * 2];
        }
    }

    @Override
    public String toString() {
        StringBuilder sb = new StringBuilder();
        sb.append(PROTOCOL_VERSION).append(",").append(getMessageType().getTypeId()).append(",").append(Message.writeString(arena));
        sb.append(",").append(players.length);
        for (String p : players) {
            sb.append(",").append(Message.writeString(p));
        }
        return sb.toString();
    }

    @Override
    public MessageType getMessageType() {
        return MessageType.MATCH_SUCCESS;
    }

}
