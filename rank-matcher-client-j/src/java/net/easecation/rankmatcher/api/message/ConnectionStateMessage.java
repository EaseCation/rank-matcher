package net.easecation.rankmatcher.api.message;

import lombok.Data;
import net.easecation.eccommons.adt.Tuple;
import net.easecation.rankmatcher.api.Message;
import net.easecation.rankmatcher.api.MessageType;

import java.util.Map;

@Data
public class ConnectionStateMessage implements Message {

    private Map<String, Tuple<String, Integer>> playerInfo;

    @Override
    public MessageType getMessageType() {
        return MessageType.CONNECTION_STATE;
    }

    public static ConnectionStateMessage of(Map<String, Tuple<String, Integer>> playerInfo) {
        ConnectionStateMessage message = new ConnectionStateMessage();
        message.playerInfo = playerInfo;
        return message;
    }

    @Override
    public void decode(String[] data) {
        /*
        self.inner.push_back(',');
        self.inner.push_back('6');
        self.write_number(player_info.len() as u64);
        for info in player_info {
            let player = info.key();
            let (arena, num_matched) = info.value();
            self.write_string(player);
            self.write_string(arena);
            self.write_number(*num_matched);
        }
        */
        // protocol(0), type(1), len(2), player_len(3), player(4), arena_len(5), arena(6), num_matched(7), ...
        int len = Integer.parseInt(data[2]);
        for (int i = 0; i < len; i++) {
            String player = data[4 + i * 5];
            String arena = data[4 + 2 + i * 5];
            int num_matched = Integer.parseInt(data[4 + 3 + i * 5]);
            playerInfo.put(player, Tuple.of(arena, num_matched));
        }
    }

    @Override
    public String toString() {
        StringBuilder sb = new StringBuilder();
        sb.append(PROTOCOL_VERSION).append(",").append(getMessageType().getTypeId()).append(",").append(playerInfo.size());
        for (Map.Entry<String, Tuple<String, Integer>> entry : playerInfo.entrySet()) {
            sb.append(",").append(Message.writeString(entry.getKey()));
            sb.append(",").append(Message.writeString(entry.getValue().getFirst()));
            sb.append(",").append(entry.getValue().getSecond());
        }
        return sb.toString();
    }

}
