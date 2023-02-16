package net.easecation.rankmatcher.api.message;

import net.easecation.eccommons.adt.Tuple;
import net.easecation.rankmatcher.api.Message;
import net.easecation.rankmatcher.api.MessageType;

import java.util.ArrayList;
import java.util.List;

public class MatchFailureMessage implements Message {

    private String arena;
    private List<Tuple<String, Integer>> players;
    //private String reason;

    @Override
    public MessageType getMessageType() {
        return MessageType.MATCH_FAILURE;
    }

    @Override
    public void decode(String[] data) {
        // protocol 0, type 1, len_arena 2, str_arena 3, num_players 4, len_player1 5, str_player1 6, rank_player1 7, len_player2 8, str_player2 9, rank_player2, ...
        arena = data[3];  // 2 3
        int numPlayers = Integer.parseInt(data[4]);
        players = new ArrayList<>();
        for (int i = 0; i < numPlayers; i++) {
            players.add(Tuple.of(
                    data[5 + 1 + i * 3],
                    Integer.parseInt(data[5 + 2 + i * 3])
            ));
        }
    }

    @Override
    public String toString() {
        StringBuilder sb = new StringBuilder();
        sb.append(PROTOCOL_VERSION).append(",").append(getMessageType().getTypeId()).append(",").append(Message.writeString(arena));
        sb.append(",").append(players.size());
        for (Tuple<String, Integer> player : players) {
            sb.append(",").append(Message.writeString(player.getFirst()));
            sb.append(",").append(player.getSecond());
        }
        return sb.toString();
    }

    public String getArena() {
        return arena;
    }

    public List<Tuple<String, Integer>> getPlayers() {
        return players;
    }
}
