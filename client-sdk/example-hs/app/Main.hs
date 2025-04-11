{-# LANGUAGE OverloadedStrings #-}

module Main where

import Data.Aeson (ToJSON, object, (.=))
import Data.Aeson.Types (Value (..))
import qualified Data.ByteString.Lazy.Char8 as L8
import Network.HTTP.Simple


paymentsAuthorizeRequest :: Value
paymentsAuthorizeRequest =
    object
        [ "amount" .= Number 1000
        , "currency" .= (145 :: Int) -- USD
        , "connector" .= (2 :: Int) -- Adyen
        , "auth_creds"
            .= object
                [ "auth_details"
                    .= object
                        [ "SignatureKey"
                            .= object
                                [ "api_key" .= String "AQEqhmfxK43MaR1Hw0m/n3Q5qf3VYp5eHZJTfEA0SnT87rrwTHXDVGtJ+kfCEMFdWw2+5HzctViMSCJMYAc=-3z4LNx7vk7s3KOZ6PMgPPx2HKrP9XfzQWMxU0OtdxTc=-suZ%I8NM9qv+?up}"
                                , "key1" .= String "JuspayDEECOM"
                                , "api_secret" .= String "AQEzgmDBbd+uOlwd9n6PxDJo8rXOaKhCAINLVnwY7G24jmdSuuL0Salp1G0BJE6opzqZqP6rEMFdWw2+5HzctViMSCJMYAc=-bn/JeFXqIxxfhhy67PE2sTZctbqzqe+fU0JprcbCEmE=-:M><zzc+t9Ne#2eb"
                                ]
                        ]
                ]
        , "payment_method" .= (0 :: Int) -- Card
        , "payment_method_data"
            .= object
                [ "data"
                    .= object
                        [ "Card"
                            .= object
                                [ "card_number" .= String "4111111111111111"
                                , "card_exp_month" .= String "03"
                                , "card_exp_year" .= String "2030"
                                , "card_cvc" .= String "737"
                                ]
                        ]
                ]
        , "address" .= object [] -- Default empty address
        , "auth_type" .= (0 :: Int) -- ThreeDS
        , "connector_request_reference_id" .= String "ref_12345"
        , "enrolled_for_3ds" .= Bool True
        , "request_incremental_authorization" .= Bool False
        , "minor_amount" .= Number 1000
        ]

paymentAuthorize :: (ToJSON a) => String -> a -> IO L8.ByteString
paymentAuthorize host req = do
    initialRequest <- parseRequest url
    let request =
            setRequestMethod "POST" $
                setRequestBodyJSON req $
                    setRequestHeader "Content-Type" ["application/json"] initialRequest
    response <- httpLBS request
    return $ getResponseBody response
  where
    url = host ++ "/ucs.payments.PaymentService/PaymentAuthorize"

main :: IO ()
main = do
    let host = "http://localhost:8000"
    response <- paymentAuthorize host paymentsAuthorizeRequest
    L8.putStrLn response
