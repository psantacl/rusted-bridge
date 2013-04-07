(ns rusted-bridge.example1
  (:require [rusted-bridge.server :as server]
            [rusted-bridge.commands :as commands]))

(defn -main [& args]
  (println "what is your name?") 
  (let [name (read-line)]
    (println (format "hi there %s" name))))


(comment
  (def server (server/start-bridge :dispatch-fn -main))
  (server/stop-server server)
  
)












