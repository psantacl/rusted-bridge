(ns rusted-bridge.commands
  (:require
   [clojure.data.json :as json])
  (:use
   [clj-etl-utils.lang-utils :only [raise]]))

(def registry (atom {} ))

(defmacro def-bridge [route desc & body]
  `(swap! registry
          assoc
          ~route
          {:handler (fn [] ~@body)
           :desc ~desc}))

(defn match-cmd? [^String incoming-cmd ^String registered-cmd]
  (loop [[part & parts]   (.split incoming-cmd "/")
         [xpart & xparts] (.split registered-cmd "/")
         binds            {}]
    (cond (and (nil? part) (nil? xpart))
          [true binds]

          (or (nil? part) (nil? xpart))
          [false {}]

          (.startsWith xpart ":")
          (recur parts xparts (assoc binds xpart part))
          
          (= part xpart)
          (recur parts xparts binds)

          :part!=xpart
          [false {}])))

(def ^{:dynamic true} binds nil)

(defn display-usage [incoming-cmd]
  (with-out-str
    (println
     (str
      (format "unrecognized command(%s)\n" incoming-cmd)
      (clojure.string/join
       "\n\n"
       (map (fn [[k v]]     
              (format "%s\n\t%s" k (:desc  v)))
            @registry))))))

(defn dispatch-command [^String incoming-cmd]
  (let [incoming-cmd    (if (.startsWith  incoming-cmd "/")
                          (.substring  incoming-cmd 1)
                          incoming-cmd)]
    
   (loop [[registered-cmd & registered-cmds] (keys @registry)
          [matches? binds] (match-cmd? incoming-cmd  registered-cmd )]
     (cond
       (and (not matches?) (empty? registered-cmds))
       (display-usage incoming-cmd)
      
       matches?
       (binding [binds binds]
         ((get-in  @registry [registered-cmd :handler])))

       :else
       (recur registered-cmds (match-cmd? incoming-cmd (first registered-cmds)))))))

(comment
  
  (def-bridge "aws/elb/:elb/ls/:instance"
      "display information about an instance for a given elb"    
    (format "1. %s" binds))

  (def-bridge "aws/elb/:elb/ls"
      "display information about an elb"
    (format "2: %s" binds))
    
  registry
  (reset! registry {})

  (dispatch-command "aws/elb/chicken-balancer/ls/happy-node")
  (dispatch-command "aws/elb/chicken-balancer/ls")
  
  (dispatch-command "command/bad")

  )
