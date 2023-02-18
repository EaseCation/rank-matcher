package net.easecation.rankmatcher.api.message;

import lombok.Data;
import net.easecation.eccommons.adt.Tuple;
import net.easecation.rankmatcher.api.CharReader;
import net.easecation.rankmatcher.api.Message;
import net.easecation.rankmatcher.api.MessageType;

import java.util.ArrayList;
import java.util.List;

@Data
public class MatchSuccessMessage implements Message {

    private String arena;
    private int stageRequestId;
    private List<Tuple<String, Integer>> players = new ArrayList<>();

    @Override
    public void decode(CharReader reader) {
        /*
        let arena = self.read_string();
        let stage_request_id = self.read_number();
        let number = self.read_number();
        let mut players = Vec::with_capacity(number as usize);
        for _ in 0..number {
            let player = self.read_string();
            let length = self.read_number();
            players.push((player, length));
        }
        Ok(Packet::MatchSuccess {
            arena,
            stage_request_id,
            players,
        })
         */
        arena = reader.readString();
        stageRequestId = reader.readNumber();
        int number = reader.readNumber();
        for (int i = 0; i < number; i++) {
            String player = reader.readString();
            int length = reader.readNumber();
            players.add(Tuple.of(player, length));
        }
    }

    @Override
    public String toString() {
        StringBuilder sb = new StringBuilder();
        sb.append(PROTOCOL_VERSION).append(",").append(getMessageType().getTypeId()).append(",").append(Message.writeString(arena));
        sb.append(",").append(stageRequestId);
        sb.append(",").append(players.size());
        for (Tuple<String, Integer> player : players) {
            sb.append(",").append(Message.writeString(player.getFirst()));
            sb.append(",").append(player.getSecond());
        }
        return sb.toString();
    }

    @Override
    public MessageType getMessageType() {
        return MessageType.MATCH_SUCCESS;
    }

}
