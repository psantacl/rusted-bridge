(ns rusted-bridge.server
  (:require
   [clojure.data.json      :as json]
   [rusted-bridge.commands :as commands])
  (:use
   [clj-etl-utils.lang-utils :only [raise]])  
  (:import
   [org.jboss.netty.bootstrap ServerBootstrap]
   [org.jboss.netty.channel.socket.nio NioServerSocketChannelFactory]  
   [org.jboss.netty.channel ChannelPipelineFactory Channels SimpleChannelUpstreamHandler ChannelFutureListener]
   [org.jboss.netty.buffer ChannelBuffers]
   [org.jboss.netty.handler.codec.frame FrameDecoder]
   [java.net InetSocketAddress]
   [java.util.concurrent Executors]))


(def config {:port 9000})

(defn make-handler [dispatch-fn]
  (proxy [SimpleChannelUpstreamHandler] []
    (channelConnected [ctx e])
    (messageReceived [ctx e]      
      (let [msg (.getMessage e)
            write-future (-> (.getChannel ctx)                             
                             (.write
                              (ChannelBuffers/copiedBuffer
                               (with-out-str
                                 (commands/dispatch-command msg dispatch-fn))
                               (java.nio.charset.Charset/forName "UTF-8"))))]
        (.addListener write-future (proxy [ChannelFutureListener] []
                                     (operationComplete [future]
                                       (-> (.getChannel future)
                                           (.disconnect)))))))
       
    (exceptionCaught [ctx ex]
      (let [stack-trac  (with-out-str
                          (.printStackTrace (.getCause ex) (java.io.PrintWriter. *out*)))
            write-future (-> (.getChannel ctx)                             
                             (.write
                              (ChannelBuffers/copiedBuffer
                               stack-trac
                               (java.nio.charset.Charset/forName "UTF-8"))))]
        (.addListener write-future (proxy [ChannelFutureListener] []
                                     (operationComplete [future]
                                       (-> (.getChannel future)
                                           (.disconnect)))))))
    
    
    (channelDisconnected [ctx e])))


(comment
  ;;send *out* over socket
  (binding [*out*  (proxy [java.io.PrintWriter] [System/out]
                     (write [obj]
                       (.write
                        *chicken*
                        (ChannelBuffers/copiedBuffer
                         obj
                         (java.nio.charset.Charset/forName "UTF-8")))
                       #_(proxy-super write obj)))]
    (println 33 ))

  )

(defn make-decoder []
  (proxy [FrameDecoder] []
    (decode [ctx channel buffer]
      (let [bytes       (.readBytes buffer  (.readableBytes buffer))
            cmd-json    (.toString bytes (java.nio.charset.Charset/forName "UTF-8"))]
        (try
          (json/read-str cmd-json)
          (catch Exception ex
            nil))))))

(defn valid-keys? [params key-lists]
  (some  (fn [key-list]
           (= (set key-list)
              (set (keys params))))
         key-lists))

(defn start-server [dispatch-fn]
  (let [ch-factory (NioServerSocketChannelFactory. (Executors/newCachedThreadPool)
                                                   (Executors/newCachedThreadPool))
        bootstrap       (ServerBootstrap. ch-factory)
        pl-factory      (reify ChannelPipelineFactory
                          (getPipeline [this]
                            (doto (Channels/pipeline)
                              (.addLast "decoder"  (make-decoder))
                              (.addLast "handler"  (make-handler dispatch-fn)))))]
    (.setPipelineFactory bootstrap pl-factory)
    (.setOption bootstrap "child.tcpNoDelay" true)
    (.setOption bootstrap "child.keepAlive" true)
    (.bind bootstrap (InetSocketAddress. (:port config)) )))

(defn start-bridge [& opts]
  (let [opts      (apply array-map opts)
        keySet    [#{:dispatch-fn}]]
    (if-not (valid-keys? opts keySet)
      (println "error: unrecognized options(%s)" opts)
      (do
        (start-server (:dispatch-fn opts))))))

(comment


  
  (rusted-bridge.commands/def-bridge "chickens"
    "list all chickens"    
    (println *chickens*))
  
  (rusted-bridge.commands/def-bridge "chicken/:name"
    "list infor about a chicken chickens"    
    (println (get *chickens* (keyword (:name rusted-bridge.commands/binds)))))

  ()

  )
