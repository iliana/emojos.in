import { App, Duration, Stack } from 'monocdk';
import { DomainName, HttpApi } from 'monocdk/aws-apigatewayv2';
import { LambdaProxyIntegration } from 'monocdk/aws-apigatewayv2-integrations';
import { Certificate } from 'monocdk/aws-certificatemanager';
import { PolicyStatement } from 'monocdk/aws-iam';
import { Runtime } from 'monocdk/aws-lambda';
import { PythonFunction } from 'monocdk/aws-lambda-python';
import { RetentionDays } from 'monocdk/aws-logs';

class EmojosStack extends Stack {
  constructor(app: App, id: string) {
    super(app, id);

    const handler = new PythonFunction(this, 'App', {
      entry: 'app',
      index: 'emojos.py',
      handler: 'handle_request',
      runtime: Runtime.PYTHON_3_8,

      environment: {
        STRIP_STAGE_PATH: 'yes',
      },

      memorySize: 256,
      timeout: Duration.seconds(15),

      logRetention: RetentionDays.ONE_MONTH,
    });

    // allow function to redirect to its own source code
    handler.addToRolePolicy(new PolicyStatement({
      actions: ['lambda:GetFunction'],
      resources: ['*'],
    }));

    new HttpApi(this, 'HttpApi', {
      defaultIntegration: new LambdaProxyIntegration({ handler }),
      defaultDomainMapping: {
        domainName: new DomainName(this, 'HttpApiDomain', {
          domainName: 'emojos.in',
          certificate: Certificate.fromCertificateArn(this, 'Certificate',
            'arn:aws:acm:us-west-2:583422757513:certificate/85544743-8592-4580-a25f-27f66fc90935'),
        }),
      },
    });
  }
}

const app = new App();
new EmojosStack(app, 'EmojosStack');
app.synth();
