package net.easecation.rankmatcher;

import io.netty.bootstrap.Bootstrap;
import io.netty.channel.*;
import io.netty.channel.nio.NioEventLoopGroup;
import io.netty.channel.socket.nio.NioSocketChannel;
import io.netty.handler.codec.http.DefaultHttpHeaders;
import io.netty.handler.codec.http.HttpClientCodec;
import io.netty.handler.codec.http.HttpObjectAggregator;
import io.netty.handler.codec.http.websocketx.WebSocketClientHandshakerFactory;
import io.netty.handler.codec.http.websocketx.WebSocketClientProtocolHandler;
import io.netty.handler.codec.http.websocketx.WebSocketFrameAggregator;
import io.netty.handler.codec.http.websocketx.WebSocketVersion;
import lombok.extern.log4j.Log4j2;
import net.easecation.rankmatcher.api.Message;
import net.easecation.rankmatcher.api.MessageReceiver;
import net.easecation.rankmatcher.api.MessageSender;
import net.easecation.rankmatcher.network.MessageCodec;
import net.easecation.rankmatcher.network.MessageHandler;

import java.net.URI;
import java.util.ArrayList;
import java.util.List;

@Log4j2
public class RankMatcherClient {

    private final EventLoopGroup loopGroup = new NioEventLoopGroup();
    private Channel channel = null;
    private boolean isHandshake = false;
    private MessageSender sender;
    private final MessageReceiver receiver;
    private final URI websocketURI;
    /**
     * 在连接成功前，
     */
    private final List<Message> initChannelMessages = new ArrayList<>();

    /*
     * name 用与 向服务端发起1h握手协议时必须带的参数
     * */
    public RankMatcherClient(String name, URI websocketURI) {
        if (name == null || name.isEmpty()) throw new IllegalArgumentException("带个 name 参数啊");
        this.name = name;
        if (websocketURI == null) {
            throw new IllegalArgumentException("url不能为空");
        }

        this.websocketURI = websocketURI;
        this.receiver = new MessageReceiver(this);
    }

    private String name;

    String getName() {
        return name;
    }

    public MessageSender getSender() {
        return sender;
    }

    public MessageReceiver getReceiver() {
        return receiver;
    }

    public boolean isHandshake() {
        return isHandshake;
    }

    public void setHandshake(boolean handshake) {
        isHandshake = handshake;
    }

    public List<Message> getInitChannelMessages() {
        return initChannelMessages;
    }

    public void start() throws Exception {
        Bootstrap bootstrap = new Bootstrap();
        bootstrap
                .group(loopGroup)
                .channel(NioSocketChannel.class)
                .option(ChannelOption.RCVBUF_ALLOCATOR, new FixedRecvByteBufAllocator(1024 * 1024))
                .option(ChannelOption.CONNECT_TIMEOUT_MILLIS, 2000)
                .handler(new ChannelInitializer<NioSocketChannel>() {
                    @Override
                    protected void initChannel(NioSocketChannel channel) {
                        channel.pipeline()
                                .addLast(new HttpClientCodec())
                                .addLast(new HttpObjectAggregator(65535))
                                .addLast(new WebSocketFrameAggregator(65535))
                                .addLast(new WebSocketClientProtocolHandler(WebSocketClientHandshakerFactory.newHandshaker(websocketURI, WebSocketVersion.V13, null, true, new DefaultHttpHeaders())))
                                .addLast(new MessageCodec())
                                .addLast(new MessageHandler(RankMatcherClient.this));
                    }
                });
        try {
            this.channel = bootstrap.connect(websocketURI.getHost(), websocketURI.getPort()).sync().channel();
            this.sender = new MessageSender(this, channel);
            log.info("RankMatcher 连接成功");
        } catch (Exception e) {
            log.warn("RankMatcher 连接失败");
            log.warn(e.getMessage());
            throw e;
        }
    }

    /*
     * 关闭连接
     * */
    public boolean shutdown() {
        return loopGroup.shutdownGracefully().isSuccess();
    }

    public boolean isActive() {
        return channel != null && channel.isActive();
    }
}