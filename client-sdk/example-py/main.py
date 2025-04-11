#!/usr/bin/env python3
"""
gRPC client for PaymentService
Usage: python main.py <host_url>
"""

import sys, os
import grpc
import logging

# Setup logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)


cwd = os.getcwd()
sys.path.append(cwd + "/generated")

# We'll import these after code generation
# from payments_pb2 import PaymentsAuthorizeRequest, Currency, Connector, AuthType, PaymentMethod
# from payments_pb2 import PaymentMethodData, Card, PaymentAddress, AuthenticationType, SignatureKey
# from payment_service_pb2_grpc import PaymentServiceStub


def main():
    # Get the URL from command line arguments
    if len(sys.argv) < 2:
        logger.error(f"Usage: {sys.argv[0]} <host_url>")
        sys.exit(1)
    
    url = sys.argv[1]

    # Import the generated modules
    from payment_pb2 import (
        PaymentsAuthorizeRequest, 
        Currency, 
        Connector, 
        AuthType, 
        PaymentMethod,
        PaymentMethodData,
        Card, 
        PaymentAddress, 
        AuthenticationType, 
        SignatureKey
    )
    from payment_pb2_grpc import PaymentServiceStub
    
    try:
        # Create a gRPC channel and client
        logger.info(f"Connecting to gRPC server at {url}")
        channel = grpc.insecure_channel(url)
        client = PaymentServiceStub(channel)
        
        # Create a payment method data object with card details
        card = Card(
            card_number="4111111111111111",
            card_exp_month="03",
            card_exp_year="2030",
            card_cvc="737"
        )
        
        # Create a payment method data object
        payment_method_data = PaymentMethodData()
        payment_method_data.card.CopyFrom(card)
        
        # Create signature key for auth credentials
        signature_key = SignatureKey(
            api_key="",
            key1="",
            api_secret=""
        )
        
        # Create auth type with signature key
        auth_creds = AuthType()
        auth_creds.signature_key.CopyFrom(signature_key)
        
        # Create a payment address (empty in this case)
        address = PaymentAddress()
        
        # Create the request
        request = PaymentsAuthorizeRequest(
            amount=1000,
            currency=Currency.USD,
            connector=Connector.ADYEN,
            auth_creds=auth_creds,
            payment_method=PaymentMethod.CARD,
            payment_method_data=payment_method_data,
            address=address,
            auth_type=AuthenticationType.THREE_DS,
            connector_request_reference_id="ref_12345",
            enrolled_for_3ds=True,
            request_incremental_authorization=False,
            minor_amount=1000
        )
        
        # Send the request
        logger.info("Sending PaymentAuthorize request")
        response = client.PaymentAuthorize(request)
        
        # Print the response
        logger.info(f"Response: {response}")
        
    except grpc.RpcError as e:
        logger.error(f"RPC error: {e.code()}: {e.details()}")
    except Exception as e:
        logger.error(f"Error: {str(e)}")
    finally:
        if 'channel' in locals():
            channel.close()


if __name__ == '__main__':
    main()
