import * as cdk from 'aws-cdk-lib';
import { HttpLambdaIntegration } from 'aws-cdk-lib/aws-apigatewayv2-integrations';
import { Construct } from 'constructs';

export class InfraStack extends cdk.Stack {
    constructor(scope: Construct, id: string, props?: cdk.StackProps) {
        super(scope, id, props);

        const func = new cdk.aws_lambda.Function(this, 'di-slash-function', {
            functionName: 'di-slash-function',
            runtime: cdk.aws_lambda.Runtime.PROVIDED_AL2023,
            handler: 'notneeded',
            code: cdk.aws_lambda.Code.fromAsset('../target/lambda/di-slash/bootstrap.zip'),
            environment: {
                RUST_BACKTRACE: "1"
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
        // output the api url
        new cdk.CfnOutput(this, "api-url", {
            value: api.apiEndpoint,
        });
    }
}
