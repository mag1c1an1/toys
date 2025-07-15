 package lake;


import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.environment.StreamExecutionEnvironment;
import org.apache.flink.walkthrough.common.sink.AlertSink;
import org.apache.flink.walkthrough.common.entity.Alert;
import org.apache.flink.walkthrough.common.entity.Transaction;
import org.apache.flink.walkthrough.common.source.TransactionSource;
import org.apache.flink.api.java.DataSet;
import org.apache.flink.api.java.ExecutionEnvironment;
import org.apache.flink.api.common.functions.MapFunction;
import org.apache.flink.api.common.functions.FilterFunction;
 public class BatchJob {
    public static void main(String[] args) throws Exception {

 // 创建执行环境
        final ExecutionEnvironment env = ExecutionEnvironment.getExecutionEnvironment();

        DataSet<String> text = env.fromElements("Hello, Flink!", "Welcome to the world of big data.", "This is a sample text.");

        // 进行转换：将每行的数据转为大写
        DataSet<String> upperCased = text.map(new MapFunction<String, String>() {
            @Override
            public String map(String value) {
                return value.toUpperCase();
            }
        });

        // 过滤：排除空行
        DataSet<String> filtered = upperCased.filter(new FilterFunction<String>() {
            @Override
            public boolean filter(String value) {
                return !value.isEmpty();
            }
        });

        // 输出结果到控制台
        filtered.print();
        
        // env.execute("Flink Batch Example");
    }
        }