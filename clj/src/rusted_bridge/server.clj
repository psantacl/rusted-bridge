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


(defn make-string-writer [ctx cmd]
  (proxy [java.io.StringWriter] []
    (write [obj]
      (let [channel (.getChannel ctx)]
        (.write
         channel
         (ChannelBuffers/copiedBuffer
          (json/write-str {:command cmd
                           :payload (do
                                      (proxy-super write obj)
                                      (proxy-super toString))})
          (java.nio.charset.Charset/forName "UTF-8"))))
      (let [buffer (.getBuffer this)]
        (.delete buffer 0 (.length buffer))))))

(defn make-handler [dispatch-fn]
  (proxy [SimpleChannelUpstreamHandler] []
    (channelConnected [ctx e])
    (messageReceived [ctx e]
      (binding [*out* (make-string-writer ctx "std-out")
                *err* (make-string-writer ctx "std-err")]
        
        (let [msg     (.getMessage e)
              channel (.getChannel ctx)]
          (commands/dispatch-command msg dispatch-fn)
          (-> channel
              (.write (ChannelBuffers/EMPTY_BUFFER))
              (.addListener (ChannelFutureListener/CLOSE))))))
    (exceptionCaught [ctx ex]
      (let [stack-trace  (with-out-str
                           (.printStackTrace (.getCause ex)
                                             (java.io.PrintWriter. *out*)))
            channel          (.getChannel ctx)]
        (-> channel
            (.write 
             (ChannelBuffers/copiedBuffer
              (json/write-str {:command "std-err"
                               :payload stack-trace})
              (java.nio.charset.Charset/forName "UTF-8"))))
        (-> channel
              (.write (ChannelBuffers/EMPTY_BUFFER))
              (.addListener (ChannelFutureListener/CLOSE)))))
    
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
