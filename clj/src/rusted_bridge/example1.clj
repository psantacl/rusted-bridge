(ns rusted-bridge.example1
  (:require [rusted-bridge.server :as server]
            [rusted-bridge.commands :as commands]))

(defn -main [& args]
  (println "in main:" args))

(comment
  
  (def server (server/start-bridge :dispatch-fn -main))
  (.close server)

  (commands/def-bridge "chicken" "look at the chickens"
    (println "chickens!"))
  )


