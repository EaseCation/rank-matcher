package net.easecation.rankmatcher.api.message;

import net.easecation.rankmatcher.api.Message;
import net.easecation.rankmatcher.api.MessageType;

public class MatchFailureMessage implements Message {

    private String arena;
    private String[] players;
    //private String reason;

    @Override
    public MessageType getMessageType() {
        return MessageType.MATCH_FAILURE;
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

    public String getArena() {
        return arena;
    }

    public String[] getPlayers() {
        return players;
    }

}
