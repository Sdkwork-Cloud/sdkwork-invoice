#!/usr/bin/env node

import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const generatorBin = path.resolve(root, '..', 'sdkwork-sdk-generator', 'bin', 'sdkgen.js');
const checkMode = process.argv.includes('--check');
const OWNER = 'sdkwork-invoice';
const authority = 'sdkwork-invoice-app-api';
const family = 'sdkwork-invoice-app-sdk';
const routeCrate = 'sdkwork-routes-invoice-app-api';

const pageParameters = [
  { name: 'page', in: 'query', required: false, schema: { type: 'integer', minimum: 1, default: 1 } },
  { name: 'page_size', in: 'query', required: false, schema: { type: 'integer', minimum: 1, maximum: 200, default: 20 } },
];
const invoiceId = { name: 'invoiceId', in: 'path', required: true, schema: { type: 'string', minLength: 1 } };
const commandHeaders = [
  { name: 'Idempotency-Key', in: 'header', required: true, schema: { type: 'string', minLength: 1, maxLength: 200 } },
  { name: 'Sdkwork-Request-Hash', in: 'header', required: true, schema: { type: 'string', minLength: 1, maxLength: 1000 } },
  { name: 'Sdkwork-Request-No', in: 'header', required: false, schema: { type: 'string', minLength: 1, maxLength: 200 } },
];

const operations = [
  {
    method: 'get', path: '/app/v3/api/invoices', operationId: 'invoices.list',
    permission: 'commerce.invoices.read', parameters: [
      { name: 'status', in: 'query', required: false, schema: { type: 'string' } }, ...pageParameters,
    ], response: 'InvoicePageResponse', summary: 'List invoices owned by the current user.',
  },
  {
    method: 'post', path: '/app/v3/api/invoices', operationId: 'invoices.create',
    permission: 'commerce.invoices.manage', parameters: commandHeaders,
    body: 'CreateInvoiceRequest', response: 'InvoiceMutationResourceResponse', status: '201',
    summary: 'Create an invoice draft.',
  },
  {
    method: 'get', path: '/app/v3/api/invoices/mine', operationId: 'invoices.mine.list',
    permission: 'commerce.invoices.read', parameters: [
      { name: 'status', in: 'query', required: false, schema: { type: 'string' } }, ...pageParameters,
    ], response: 'InvoicePageResponse', summary: 'List invoices owned by the current user.',
  },
  {
    method: 'get', path: '/app/v3/api/invoices/statistics', operationId: 'invoices.statistics.retrieve',
    permission: 'commerce.invoices.read', response: 'InvoiceStatisticsResourceResponse',
    summary: 'Retrieve aggregate invoice statistics for the current user.',
  },
  {
    method: 'get', path: '/app/v3/api/invoices/{invoiceId}', operationId: 'invoices.retrieve',
    permission: 'commerce.invoices.read', parameters: [invoiceId], response: 'InvoiceResourceResponse',
    summary: 'Retrieve an invoice owned by the current user.',
  },
  {
    method: 'patch', path: '/app/v3/api/invoices/{invoiceId}', operationId: 'invoices.update',
    permission: 'commerce.invoices.manage', parameters: [invoiceId, ...commandHeaders],
    body: 'UpdateInvoiceRequest', response: 'InvoiceMutationResourceResponse',
    summary: 'Update an invoice draft.',
  },
  {
    method: 'get', path: '/app/v3/api/invoices/{invoiceId}/items', operationId: 'invoices.items.list',
    permission: 'commerce.invoices.read', parameters: [invoiceId, ...pageParameters],
    response: 'InvoiceItemPageResponse', summary: 'List items for an invoice owned by the current user.',
  },
  {
    method: 'post', path: '/app/v3/api/invoices/{invoiceId}/submissions', operationId: 'invoices.submissions.create',
    permission: 'commerce.invoices.manage', parameters: [invoiceId, ...commandHeaders],
    response: 'InvoiceMutationResourceResponse', status: '201', summary: 'Submit an invoice draft.',
  },
  {
    method: 'post', path: '/app/v3/api/invoices/{invoiceId}/cancellations', operationId: 'invoices.cancellations.create',
    permission: 'commerce.invoices.manage', parameters: [invoiceId, ...commandHeaders],
    body: 'CancelInvoiceRequest', bodyRequired: false, response: 'InvoiceCommandResourceResponse', status: '201',
    summary: 'Cancel an invoice.',
  },
];

function stableJson(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

function envelope(dataSchema) {
  return {
    type: 'object', required: ['code', 'data', 'traceId'],
    properties: {
      code: { type: 'integer', format: 'int32', const: 0 }, data: dataSchema,
      traceId: { type: 'string', format: 'uuid' },
    },
  };
}

function resourceEnvelope(itemRef) {
  return envelope({ type: 'object', required: ['item'], properties: { item: { $ref: itemRef } } });
}

function pageEnvelope(itemRef) {
  return envelope({
    type: 'object', required: ['items', 'pageInfo'],
    properties: {
      items: { type: 'array', items: { $ref: itemRef } },
      pageInfo: { $ref: '#/components/schemas/PageInfo' },
    },
  });
}

function schemas() {
  return {
    SdkWorkApiResponse: envelope({}),
    PageInfo: {
      type: 'object', required: ['mode'], properties: {
        mode: { type: 'string', enum: ['offset', 'cursor'] }, page: { type: ['integer', 'null'] },
        pageSize: { type: ['integer', 'null'] }, totalItems: { type: ['string', 'null'] },
        totalPages: { type: ['integer', 'null'] }, nextCursor: { type: ['string', 'null'] },
        hasMore: { type: ['boolean', 'null'] },
      },
    },
    ProblemDetail: {
      type: 'object', required: ['type', 'title', 'status', 'detail', 'code', 'traceId'],
      properties: {
        type: { type: 'string', format: 'uri-reference' }, title: { type: 'string' },
        status: { type: 'integer', format: 'int32' }, detail: { type: 'string' },
        code: { type: 'integer', format: 'int32' }, traceId: { type: 'string', format: 'uuid' },
        instance: { type: ['string', 'null'] }, operationId: { type: ['string', 'null'] },
      },
    },
    InvoiceItem: {
      type: 'object', required: ['id', 'invoiceId', 'title', 'amount', 'taxAmount', 'createdAt'],
      properties: {
        id: { type: 'string' }, invoiceId: { type: 'string' }, orderItemId: { type: ['string', 'null'] },
        title: { type: 'string' }, amount: { type: 'string' }, taxAmount: { type: 'string' },
        createdAt: { type: 'string' },
      }, additionalProperties: false,
    },
    Invoice: {
      type: 'object', required: ['id', 'orderId', 'paymentId', 'titleId', 'status', 'createdAt', 'updatedAt', 'totalAmount', 'items'],
      properties: {
        id: { type: 'string' }, orderId: { type: 'string' }, paymentId: { type: 'string' },
        titleId: { type: 'string' }, status: { type: 'string' }, invoiceNo: { type: ['string', 'null'] },
        invoiceCode: { type: ['string', 'null'] }, documentUrl: { type: ['string', 'null'], format: 'uri' },
        createdAt: { type: 'string' }, issuedAt: { type: ['string', 'null'] }, updatedAt: { type: 'string' },
        totalAmount: { type: 'string' }, items: { type: 'array', items: { $ref: '#/components/schemas/InvoiceItem' } },
      }, additionalProperties: false,
    },
    InvoiceMutation: {
      type: 'object', required: ['invoiceId', 'status', 'title', 'titleType', 'totalAmount', 'type', 'createdAt', 'updatedAt'],
      properties: {
        invoiceId: { type: 'string' }, status: { type: 'string' }, title: { type: 'string' },
        titleType: { type: 'string' }, totalAmount: { type: 'string' }, type: { type: 'string' },
        createdAt: { type: 'string' }, updatedAt: { type: 'string' },
      }, additionalProperties: false,
    },
    InvoiceStatistics: {
      type: 'object', required: ['totalInvoices', 'pendingInvoices', 'issuedInvoices', 'cancelledInvoices'],
      properties: {
        totalInvoices: { type: 'integer', format: 'int64' }, pendingInvoices: { type: 'integer', format: 'int64' },
        issuedInvoices: { type: 'integer', format: 'int64' }, cancelledInvoices: { type: 'integer', format: 'int64' },
      }, additionalProperties: false,
    },
    CreateInvoiceRequest: {
      type: 'object', required: ['title'], properties: {
        title: { type: 'string', minLength: 1 }, taxNo: { type: ['string', 'null'] },
        titleType: { type: ['string', 'null'] }, totalAmount: { oneOf: [{ type: 'string' }, { type: 'number' }] },
        type: { type: ['string', 'null'] },
      }, additionalProperties: false,
    },
    UpdateInvoiceRequest: {
      type: 'object', minProperties: 1, properties: {
        bankAccount: { type: ['string', 'null'] }, bankName: { type: ['string', 'null'] },
        registerAddress: { type: ['string', 'null'] }, registerPhone: { type: ['string', 'null'] },
        taxNo: { type: ['string', 'null'] }, title: { type: ['string', 'null'] },
      }, additionalProperties: false,
    },
    CancelInvoiceRequest: {
      type: 'object', properties: { cancelReason: { type: ['string', 'null'] } }, additionalProperties: false,
    },
    InvoicePageResponse: pageEnvelope('#/components/schemas/Invoice'),
    InvoiceItemPageResponse: pageEnvelope('#/components/schemas/InvoiceItem'),
    InvoiceResourceResponse: resourceEnvelope('#/components/schemas/Invoice'),
    InvoiceMutationResourceResponse: resourceEnvelope('#/components/schemas/InvoiceMutation'),
    InvoiceStatisticsResourceResponse: resourceEnvelope('#/components/schemas/InvoiceStatistics'),
    InvoiceCommand: {
      type: 'object', required: ['accepted'], properties: {
        accepted: { type: 'boolean' }, resourceId: { type: ['string', 'null'] }, status: { type: ['string', 'null'] },
      },
    },
    InvoiceCommandResourceResponse: resourceEnvelope('#/components/schemas/InvoiceCommand'),
  };
}

function problemResponse(description) {
  return { description, content: { 'application/problem+json': { schema: { $ref: '#/components/schemas/ProblemDetail' } } } };
}

function operationObject(entry) {
  const successStatus = entry.status ?? '200';
  const operation = {
    tags: ['invoices'], summary: entry.summary, operationId: entry.operationId,
    parameters: entry.parameters ?? [],
    responses: {
      [successStatus]: {
        description: successStatus === '201' ? 'Created' : 'Successful response',
        content: { 'application/json': { schema: { $ref: `#/components/schemas/${entry.response}` } } },
      },
      '400': problemResponse('Invalid request'), '401': problemResponse('Authentication required'),
      '403': problemResponse('Permission denied'), '404': problemResponse('Resource not found'),
      '409': problemResponse('Resource conflict'), '500': problemResponse('Internal server error'),
    },
    security: [{ AuthToken: [], AccessToken: [] }],
    'x-sdkwork-owner': OWNER, 'x-sdkwork-api-authority': authority,
    'x-sdkwork-domain': 'commerce', 'x-sdkwork-resource': entry.operationId.split('.').slice(0, -1).join('.'),
    'x-sdkwork-request-context': 'WebRequestContext', 'x-sdkwork-api-surface': 'app-api',
    'x-sdkwork-server-request-id': true, 'x-sdkwork-source-route-crate': routeCrate,
    'x-sdkwork-source': `${routeCrate}:${entry.path}`, 'x-sdkwork-auth-mode': 'dual-token',
    'x-sdkwork-permission': entry.permission,
  };
  if (entry.body) {
    operation.requestBody = {
      required: entry.bodyRequired ?? true,
      content: { 'application/json': { schema: { $ref: `#/components/schemas/${entry.body}` } } },
    };
  }
  return operation;
}

function document() {
  const paths = {};
  for (const entry of operations) {
    paths[entry.path] ??= {};
    paths[entry.path][entry.method] = operationObject(entry);
  }
  return {
    openapi: '3.1.2',
    info: {
      title: 'SDKWork Invoice App API', version: '1.0.0',
      description: 'Owner-only invoice app API contract.',
      'x-sdkwork-api-authority': authority, 'x-sdkwork-sdk-family': family, 'x-sdkwork-owner': OWNER,
    },
    servers: [{ url: 'http://127.0.0.1:8080', description: 'Local SDKWork gateway' }],
    tags: [{ name: 'invoices', description: 'Invoice resources.', 'x-sdk-nested-resource-surface': true }],
    security: [{ AuthToken: [], AccessToken: [] }], paths,
    components: {
      securitySchemes: {
        AuthToken: { type: 'http', scheme: 'bearer', bearerFormat: 'JWT' },
        AccessToken: { type: 'apiKey', in: 'header', name: 'Access-Token' },
      },
      schemas: schemas(),
    },
  };
}

function routeManifest() {
  return {
    schemaVersion: 1, kind: 'sdkwork.route.manifest', packageName: routeCrate,
    surface: 'app-api', owner: OWNER, domain: 'commerce', capability: 'invoice',
    apiAuthority: authority, sdkFamily: family, prefix: '/app/v3/api',
    source: { crateRoot: `crates/${routeCrate}`, crateImport: 'sdkwork_routes_invoice_app_api' },
    routes: operations.map((entry) => ({
      method: entry.method.toUpperCase(), path: entry.path, operationId: entry.operationId,
      tags: ['invoices'], auth: { mode: 'dual-token', required: true, permission: entry.permission },
      ownership: { owner: OWNER, apiAuthority: authority }, source: { file: `${routeCrate}:${entry.path}` },
      requestContext: 'WebRequestContext', apiSurface: 'app-api',
    })),
  };
}

function synchronize(relativePath, content) {
  const targetPath = path.join(root, relativePath);
  const current = existsSync(targetPath) ? readFileSync(targetPath, 'utf8') : '';
  if (checkMode && current !== content) throw new Error(`${relativePath} is not synchronized`);
  if (!checkMode && current !== content) {
    mkdirSync(path.dirname(targetPath), { recursive: true });
    writeFileSync(targetPath, content, 'utf8');
  }
}

try {
  const content = stableJson(document());
  const familyRoot = path.join(root, 'sdks', family);
  const generatedRoot = path.join(familyRoot, `${family}-typescript`, 'generated', 'server-openapi');
  synchronize(`apis/app-api/invoice/${authority}.openapi.json`, content);
  synchronize(`sdks/${family}/openapi/${authority}.openapi.json`, content);
  synchronize(`sdks/${family}/openapi/${authority}.sdkgen.json`, content);
  synchronize(`sdks/_route-manifests/app-api/${routeCrate}.route-manifest.json`, stableJson(routeManifest()));
  const manifest = JSON.parse(readFileSync(path.join(familyRoot, 'sdk-manifest.json'), 'utf8'));
  if (manifest.sdkOwner !== OWNER || manifest.apiAuthority !== authority
      || manifest.ownerOnlyOperationCount !== operations.length) {
    throw new Error(`${family}/sdk-manifest.json does not match the authority contract`);
  }
  if (!checkMode) {
    const input = path.join(familyRoot, 'openapi', `${authority}.sdkgen.json`);
    const result = spawnSync('node', [
      generatorBin, 'generate', '--input', input, '--output', generatedRoot,
      '--name', family, '--type', 'app', '--language', 'typescript',
      '--base-url', 'http://127.0.0.1:8080', '--api-prefix', '/app/v3/api',
      '--fixed-sdk-version', '0.1.0', '--sdk-root', familyRoot, '--sdk-name', family,
      '--package-name', `${family}-generated-typescript`, '--standard-profile', 'sdkwork-v3',
    ], { cwd: familyRoot, stdio: 'inherit' });
    if (result.status !== 0) throw new Error(`${family} sdkgen failed with exit code ${result.status}`);
  } else if (!existsSync(path.join(generatedRoot, 'src', 'index.ts'))) {
    throw new Error(`${family} generated TypeScript transport is missing`);
  }
  process.stdout.write(`[invoice_api_sdk_generate] ${checkMode ? 'check passed' : 'generation completed'} (${operations.length} operations)\n`);
} catch (error) {
  process.stderr.write(`[invoice_api_sdk_generate] ${error instanceof Error ? error.message : String(error)}\n`);
  process.exit(1);
}
