(ns rusted-bridge.core
  (:require
   [clojure.data.json :as json])
  (:use
   [clj-etl-utils.lang-utils :only [raise]])  
  (:import
   [io.netty.bootstrap ServerBootstrap]
   [io.netty.channel.socket.nio NioServerSocketChannelFactory]  
   [io.netty.channel ChannelPipelineFactory Channels SimpleChannelUpstreamHandler ChannelFutureListener]
   [io.netty.buffer ChannelBuffers]
   [io.netty.handler.codec.frame FrameDecoder]
   [java.net InetSocketAddress]
   [java.util.concurrent Executors]))


(def config {:port 9000})

(defn make-handler []
  (proxy [SimpleChannelUpstreamHandler] []
    (channelConnected [ctx e]
      (println "channel connected event"))
    (messageReceived [ctx e]      
      (let [msg (.getMessage e)
            _ (def *tuna* (.getChannel ctx))
            _ (def *msg* msg)
            write-future (-> (.getChannel ctx)
                             (.write (io.netty.buffer.ChannelBuffers/copiedBuffer
                                      "nice to meet you"
                                      (java.nio.charset.Charset/forName "UTF-8"))))]
        (.addListener write-future (proxy [ChannelFutureListener] []
                                       (operationComplete [future]
                                         (println "the write has completed")
                                         (-> (.getChannel future)
                                             (.disconnect)))))))
    
    (exceptionCaught [ctx ex]
      (def *ex* ex)
      (println "exception was encountered :("))
    (channelDisconnected [ctx e]
      (println "channel disconnected"))))

(defn make-decoder []
  (proxy [FrameDecoder] []
    (decode [ctx channel buffer]
      (let [bytes       (.readBytes buffer  (.readableBytes buffer))
            cmd-json    (.toString bytes (java.nio.charset.Charset/forName "UTF-8"))]
        (def *laser* cmd-json)
        (try
          (json/read-str cmd-json)
        (catch Exception ex
          nil))))))


(comment

  (def server (start-netty-server))
  
  (.close server)
  
  (.getCause  *ex*)


  )
(defn start-netty-server []
  (let [ch-factory (NioServerSocketChannelFactory. (Executors/newCachedThreadPool)
                                                   (Executors/newCachedThreadPool))
        bootstrap       (ServerBootstrap. ch-factory)
        pl-factory      (reify ChannelPipelineFactory
                          (getPipeline [this]
                            (doto (Channels/pipeline)
                              (.addLast "decoder"  (make-decoder))
                              (.addLast "handler"  (make-handler)))))]
    (.setPipelineFactory bootstrap pl-factory)
    (.setOption bootstrap "child.tcpNoDelay" true)
    (.setOption bootstrap "child.keepAlive" true)
    (.bind bootstrap (InetSocketAddress. (:port config)) )))


(comment

  (import 'io.netty.buffer.ChannelBuffers )
  
  (.write *tuna*
          (io.netty.buffer.ChannelBuffers/copiedBuffer "answer me!" (java.nio.charset.Charset/forName "UTF-8")))
  
  (.disconnect *tuna*)
  
  (.toString *msg*   (java.nio.charset.Charset/forName "UTF-8"))
  
  (def server (start-netty-server))
  
  (.close server)

  (import 'java.nio.charset.Charset)
  (.toString chicken   (java.nio.charset.Charset/forName "UTF-8"))
  
  (java.nio.charset.Charset/forName "UTF-8")
  )