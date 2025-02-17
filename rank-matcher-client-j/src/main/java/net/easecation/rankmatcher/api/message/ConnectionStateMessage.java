package net.easecation.rankmatcher.api.message;

import lombok.Data;
import net.easecation.eccommons.adt.Tuple;
import net.easecation.rankmatcher.api.CharReader;
import net.easecation.rankmatcher.api.Message;
import net.easecation.rankmatcher.api.MessageType;

import java.util.LinkedHashMap;
import java.util.Map;

@Data
public class ConnectionStateMessage implements Message {

    private Map<String, Tuple<String, Integer>> playerInfo = new LinkedHashMap<>();

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
    public void decode(CharReader reader) {
        /*
        fn read_v1_connection_state(&mut self) -> Result<Packet, PacketFormat> {
            let number = self.read_number();
            let player_info = DashMap::with_capacity(number as usize);
            for _ in 0..number {
                let player = self.read_string();
                let arena = self.read_string();
                let num_matched = self.read_number();
                player_info.insert(player, (arena, num_matched));
            }
            Ok(Packet::ConnectionState { player_info })
        }
        */
        int number = reader.readNumber();
        for (int i = 0; i < number; i++) {
            String player = reader.readString();
            String arena = reader.readString();
            int numMatched = reader.readNumber();
            playerInfo.put(player, Tuple.of(arena, numMatched));
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
