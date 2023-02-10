package net.easecation.rankmatcher.api;

import net.easecation.rankmatcher.RankMatcherClient;

import java.util.HashMap;
import java.util.Map;

/*
* 消息接收器
* */
public class MessageReceiver {

    private final Map<MessageType, MessageHandler> handlers = new HashMap<>();
    private final RankMatcherClient client;

    public MessageReceiver(RankMatcherClient client) {
        this.client = client;
    }

    public void addHandler(MessageType type, MessageHandler handler) {
        handlers.put(type, handler);
    }

    public <T extends Message> void receive(T message) {
        MessageHandler handler = handlers.get(message.getMessageType());
        if (handler != null) handler.handle(message);
    }

    public interface MessageHandler {
        <T extends Message> void handle(T message);
    }

}
