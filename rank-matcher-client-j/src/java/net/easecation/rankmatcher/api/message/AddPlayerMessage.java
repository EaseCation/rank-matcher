package net.easecation.rankmatcher.api.message;

import lombok.Data;
import net.easecation.rankmatcher.api.Message;
import net.easecation.rankmatcher.api.MessageType;

@Data
public class AddPlayerMessage implements Message {

    private String arena;
    private String player;
    private int rank;
    private int length;  // 通常是1。用于按队伍为单位匹配时，以队长的名义和分数匹配，此时length为队伍成员的数量

    public static AddPlayerMessage of(String arena, String player, int rank, int length) {
        AddPlayerMessage message = new AddPlayerMessage();
        message.arena = arena;
        message.player = player;
        message.rank = rank;
        message.length = length;
        return message;
    }

    @Override
    public MessageType getMessageType() {
        return MessageType.ADD_PLAYER;
    }

    @Override
    public String toString() {
        return PROTOCOL_VERSION + "," + getMessageType().getTypeId() + "," + Message.writeString(arena) + "," + Message.writeString(player) + "," + rank + "," + length;
    }

}
