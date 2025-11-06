use alloy::sol;

// IdentityRegistry contract events
sol! {
    #[sol(rpc)]
    contract IdentityRegistry {
        event Registered(uint256 indexed agentId, string tokenURI, address indexed owner);
        event MetadataSet(uint256 indexed agentId, string indexed indexedKey, string key, bytes value);
        event UriUpdated(uint256 indexed agentId, string newUri, address indexed updatedBy);
    }
}

// ReputationRegistry contract events
sol! {
    #[sol(rpc)]
    contract ReputationRegistry {
        event NewFeedback(
            uint256 indexed agentId,
            address indexed client,
            uint8 score,
            bytes32 indexed tag1,
            bytes32 tag2,
            string feedbackURI,
            bytes32 feedbackHash
        );
        event FeedbackRevoked(uint256 indexed agentId, address indexed client, uint256 feedbackIndex);
        event ResponseAppended(
            uint256 indexed agentId,
            address indexed client,
            uint256 feedbackIndex,
            address responder,
            string responseURI,
            bytes32 responseHash
        );
    }
}

// ValidationRegistry contract events
sol! {
    #[sol(rpc)]
    contract ValidationRegistry {
        event ValidationRequest(
            address indexed validatorAddress,
            uint256 indexed agentId,
            string requestUri,
            bytes32 indexed requestHash
        );
        event ValidationResponse(
            address indexed validatorAddress,
            uint256 indexed agentId,
            bytes32 indexed requestHash,
            uint8 response,
            string responseUri,
            bytes32 responseHash,
            bytes32 tag
        );
    }
}

// Contract modules are available via the module name directly
