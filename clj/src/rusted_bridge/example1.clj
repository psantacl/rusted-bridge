(ns rusted-bridge.example1
  (:require [rusted-bridge.server :as server]
            [rusted-bridge.commands :as commands]))

(defn -main [& args]
  (println "in main: " args))

(comment
  (apply -main "chicken of the sea")
  
  (def server (server/start-bridge :dispatch-fn -main))
  (.close server)
  
  (binding [*out* (java.io.StringWriter.)]
    (println "in main: " '("chicken"))
    (clojure.data.json/write-str (.toString  *out*)))

  (commands/def-bridge "chicken" "look at the chickens"
    (println "chickens!"))
  )



(let [chicken  (proxy [java.io.StringWriter] []
                 (write [obj]
                   (proxy-super write obj)
                   (proxy-super toString)))]
  (.write chicken "my bum hurts"))


