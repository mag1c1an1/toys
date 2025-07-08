import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.environment.StreamExecutionEnvironment;

public class WordCount {

    public static void main(String[] args) throws Exception {
        // 设置执行环境
        StreamExecutionEnvironment env = StreamExecutionEnvironment.getExecutionEnvironment();

        // 直接在代码中定义输入数据
        // DataStream<String> text = env.fromElements(
        //         "Hello Flink",
        //         "Hello World",
        //         "Flink is cool",
        //         "World is wonderful");
        // text.print();

        // 执行应用
        env.execute("WordCount Example");
    }

}