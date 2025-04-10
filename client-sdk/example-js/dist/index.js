"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
const grpc = __importStar(require("@grpc/grpc-js"));
const protoLoader = __importStar(require("@grpc/proto-loader"));
const path = __importStar(require("path"));
const winston = __importStar(require("winston"));
// Logging setup
const logger = winston.createLogger({
    level: 'info',
    format: winston.format.combine(winston.format.timestamp(), winston.format.printf(({ level, message, timestamp }) => `${timestamp} - ${level}: ${message}`)),
    transports: [new winston.transports.Console()],
});
// Paths
const PROTO_DIR = path.resolve('../../backend/grpc-api-types/proto');
const PROTO_PATH = path.join(PROTO_DIR, 'payment.proto');
async function main() {
    if (process.argv.length < 3) {
        logger.error(`Usage: ${process.argv[1]} <host_url>`);
        process.exit(1);
    }
    const url = process.argv[2];
    try {
        logger.info('Loading proto definitions...');
        const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
            keepCase: true,
            longs: String,
            enums: String,
            defaults: true,
            oneofs: true,
        });
        const protoDescriptor = grpc.loadPackageDefinition(packageDefinition);
        logger.info(`Proto Descriptor: ${JSON.stringify(protoDescriptor, null, 2)}`);
        const paymentService = protoDescriptor.ucs.payments.PaymentService;
        if (!paymentService) {
            logger.error('PaymentService is not defined in proto descriptor');
            process.exit(1);
        }
        const client = new paymentService(url, grpc.credentials.createInsecure());
        const request = {
            amount: 1000,
            currency: 'USD',
            connector: 'ADYEN',
            auth_creds: { signature_key: { api_key: '', key1: '', api_secret: '' } },
            payment_method: 'CARD',
            payment_method_data: { card: { card_number: '4111111111111111', card_exp_month: '03', card_exp_year: '2030', card_cvc: '737' } },
            address: {},
            auth_type: 'THREE_DS',
            connector_request_reference_id: 'ref_12345',
            enrolled_for_3ds: true,
            request_incremental_authorization: false,
            minor_amount: 1000,
        };
        logger.info('Sending PaymentAuthorize request...');
        client.PaymentAuthorize(request, (error, response) => {
            if (error) {
                logger.error(`RPC error: ${error.code}: ${error.details}`);
                return;
            }
            logger.info(`Response: ${JSON.stringify(response)}`);
        });
    }
    catch (error) {
        logger.error(`Error: ${error.message}`);
        if (error.stack) {
            logger.debug(`Stack: ${error.stack}`);
        }
    }
}
main().catch((err) => {
    logger.error(`Uncaught error: ${err.message}`);
    process.exit(1);
});
