package net.easecation.rankmatcher.network;

import io.netty.channel.ChannelHandlerContext;
import io.netty.channel.SimpleChannelInboundHandler;
import lombok.extern.log4j.Log4j2;
import net.easecation.rankmatcher.RankMatcherClient;
import net.easecation.rankmatcher.api.Message;

import static io.netty.handler.codec.http.websocketx.WebSocketClientProtocolHandler.ClientHandshakeStateEvent.HANDSHAKE_COMPLETE;
import static io.netty.handler.codec.http.websocketx.WebSocketClientProtocolHandler.ClientHandshakeStateEvent.HANDSHAKE_ISSUED;

/*
* 用与处理 消息接收
* */
@Log4j2
public class MessageHandler extends SimpleChannelInboundHandler<Message> {

    private final RankMatcherClient client;

    public MessageHandler(RankMatcherClient client){
        this.client = client;
    }

    @Override
    public void userEventTriggered(ChannelHandlerContext ctx, Object evt) throws Exception {
        super.userEventTriggered(ctx, evt);

        if (HANDSHAKE_ISSUED.equals(evt)) {
            log.info("正在向 Rank Matcher 发送TCP握手");
        }

        if (HANDSHAKE_COMPLETE.equals(evt)) {
            log.info("Rank Matcher TCP握手完成");
        }
    }

    protected void channelRead0(ChannelHandlerContext ctx, Message msg) {
        client.getReceiver().receive(msg);
    }

}