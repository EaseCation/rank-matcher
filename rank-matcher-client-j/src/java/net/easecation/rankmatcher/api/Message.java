package net.easecation.rankmatcher.api;

import net.easecation.rankmatcher.api.message.*;

import java.util.HashMap;
import java.util.Map;
import java.util.function.Supplier;

/*
* 封装一条 消息数据
* */
public interface Message {

    String PROTOCOL_VERSION = "1";

    /**
     * 用于解码的消息类型（所以服务端不发的，可以不注册）
     */
    Map<Integer, Supplier<Message>> MESSAGE_SUPPLIERS = new HashMap<Integer, Supplier<Message>>(){{
        put(MessageType.ADD_ARENA.getTypeId(), AddArenaMessage::new);
        put(MessageType.REMOVE_ARENA.getTypeId(), RemoveArenaMessage::new);
        put(MessageType.ADD_PLAYER.getTypeId(), AddPlayerMessage::new);
        put(MessageType.REMOVE_PLAYER.getTypeId(), RemovePlayerMessage::new);
        put(MessageType.GET_OR_SUBSCRIBE_STATE.getTypeId(), GetOrSubscribeStateMessage::new);
        put(MessageType.CONNECTION_STATE.getTypeId(), ConnectionStateMessage::new);
        put(MessageType.MATCH_SUCCESS.getTypeId(), MatchSuccessMessage::new);
        put(MessageType.MATCH_FAILURE.getTypeId(), MatchFailureMessage::new);
        put(MessageType.FORMAT_ERROR.getTypeId(), FormatErrorMessage::new);
    }};

    MessageType getMessageType();

    default void decode(CharReader reader) {}

    String toString();

    static String writeString(String string) {
        string = string.replace(",", "\\_");
        return string.length() + "," + string;
    }

}
