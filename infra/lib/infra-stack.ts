import * as cdk from 'aws-cdk-lib';
import { HttpLambdaIntegration } from 'aws-cdk-lib/aws-apigatewayv2-integrations';
import { Construct } from 'constructs';
import { config } from 'dotenv';

config();

export class InfraStack extends cdk.Stack {
    constructor(scope: Construct, id: string, props?: cdk.StackProps) {
        super(scope, id, props);

        const queue = new cdk.aws_sqs.Queue(this, "di-slash-queue", {
            deliveryDelay: cdk.Duration.minutes(1),
            queueName: "di-slash-queue",
            retentionPeriod: cdk.Duration.days(1),
        });

        const func = new cdk.aws_lambda.Function(this, 'di-slash-function', {
            functionName: 'di-slash-function',
            runtime: cdk.aws_lambda.Runtime.PROVIDED_AL2023,
            handler: 'notneeded',
            code: cdk.aws_lambda.Code.fromAsset('../target/lambda/di-slash/bootstrap.zip'),
            environment: {
                RUST_BACKTRACE: "1",
                QUEUE_URL: queue.queueUrl,
                RAIDBOTS_COOKIE: process.env.RAIDBOTS_COOKIE!,
                LIBSQL_URL: process.env.LIBSQL_URL!,
                LIBSQL_TOKEN: process.env.LIBSQL_TOKEN!,
            },
        });
        const api = new cdk.aws_apigatewayv2.HttpApi(this, `di-http-api`, {
            apiName: `di-slash-api`,
            description:
                "Bot for Dwarf Invasion Discord server Slash command integration",
            defaultIntegration: new HttpLambdaIntegration(
                "DwarfInvasionBotApiIntegration",
                func,
            ),
        });


        const queueLambda = new cdk.aws_lambda.Function(this, "di-sim-worker-lambda", {
            functionName: "di-sims-worker",
            runtime: cdk.aws_lambda.Runtime.PROVIDED_AL2023,
            handler: 'notneeded',
            code: cdk.aws_lambda.Code.fromAsset('../target/lambda/di-worker/bootstrap.zip'),
            environment: {
                RUST_BACKTRACE: "1",
                QUEUE_URL: queue.queueUrl,
            },
        });

        queue.grantSendMessages(queueLambda);
        queue.grantConsumeMessages(queueLambda);

        // output the api url
        new cdk.CfnOutput(this, "api-url", {
            value: api.apiEndpoint,
        });
    }
}
