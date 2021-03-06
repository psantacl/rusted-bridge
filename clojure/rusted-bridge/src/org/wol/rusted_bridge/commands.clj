(ns org.wol.rusted-bridge.commands
  (:require
   clojure.string
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
          (recur parts xparts (assoc binds (keyword (clojure.string/replace-first xpart ":" "")) part))
          
          (= part xpart)
          (recur parts xparts binds)

          :part!=xpart
          [false {}])))

(def ^{:dynamic true} binds nil)

(defn display-usage [incoming-cmd]
  (println
   (str
    (format "unrecognized command(%s)\n" incoming-cmd)
    (clojure.string/join
     "\n\n"
     (map (fn [[k v]]     
            (format "%s\n\t%s" k (:desc  v)))
          @registry)))))

(defn display-help []
  (println
   (clojure.string/join
    "\n\n"
    (map (fn [[k v]]     
           (format "%s\n\t%s" k (:desc  v)))
         @registry))))


(defn exec-command [^String incoming-cmd dispatch-fn]
  (loop [[registered-cmd & registered-cmds] (keys @registry)
         [matches? binds] (or (and registered-cmd
                                   (match-cmd? incoming-cmd  registered-cmd ))
                              [false nil])]      
    (cond
      (= (.toLowerCase incoming-cmd) "help")
      (display-help)

      matches?
      (binding [binds binds]
        ((get-in  @registry [registered-cmd :handler])))

      (and (not matches?) (empty? registered-cmds) (not (nil? dispatch-fn)))
      (dispatch-fn incoming-cmd)

      (and (not matches?) (empty? registered-cmds) (nil? dispatch-fn))
      (display-usage incoming-cmd)          

      :else
      (recur registered-cmds (match-cmd? incoming-cmd (first registered-cmds))))))



(comment

  ((:handler
    (get @registry "chicken/ls")))
  
  (keys @registry)
  (reset! registry {})

  (dispatch-command "aws/elb/chicken-balancer/ls/happy-node")
  (dispatch-command "aws/elb/chicken-balancer/ls")
  
  (dispatch-command "chicken/ls")
  )




