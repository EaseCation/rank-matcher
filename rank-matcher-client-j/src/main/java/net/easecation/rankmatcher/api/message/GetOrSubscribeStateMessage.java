package net.easecation.rankmatcher.api.message;

import lombok.Data;
import net.easecation.rankmatcher.api.Message;
import net.easecation.rankmatcher.api.MessageType;

@Data
public class GetOrSubscribeStateMessage implements Message {

    private int period;  // 0 => 立即返回，并且以后不再发送, 非0 => 每隔多少秒返回一次

    public static GetOrSubscribeStateMessage of(int period) {
        GetOrSubscribeStateMessage message = new GetOrSubscribeStateMessage();
        message.period = period;
        return message;
    }

    @Override
    public MessageType getMessageType() {
        return MessageType.GET_OR_SUBSCRIBE_STATE;
    }

    @Override
    public String toString() {
        return PROTOCOL_VERSION + "," + getMessageType().getTypeId() + "," + period;
    }
}
