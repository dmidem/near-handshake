use std::io;

use tokio::{io::AsyncSeekExt, time::Duration};

use near_crypto::{ED25519PublicKey, PublicKey};

use near_primitives::{
    block::GenesisId, hash::CryptoHash, network::PeerId, version::PROTOCOL_VERSION,
};

use crate::{
    network_protocol::{Handshake, HandshakeFailure, HandshakeResponse, MessageType, NetworkError},
    Connection,
};

type TestConnection = Connection<io::Cursor<Vec<u8>>>;

async fn seek_to_start(connection: &mut TestConnection) {
    connection
        .stream
        .seek(io::SeekFrom::Start(0))
        .await
        .unwrap();
}

async fn assert_end_stream(connection: &mut TestConnection) {
    assert_eq!(
        connection.stream.get_ref().len() as u64,
        connection
            .stream
            .seek(io::SeekFrom::Current(0))
            .await
            .unwrap()
    );
}

async fn test_handshake_request(
    connection: &mut TestConnection,
) -> Result<HandshakeResponse, NetworkError> {
    seek_to_start(connection).await;

    let handshake_response = connection
        .handshake_with_optional_genesis(PROTOCOL_VERSION, None, 0)
        .await;

    assert_end_stream(connection).await;

    handshake_response
}

fn assert_unexpected_eof(handshake_response: Result<HandshakeResponse, NetworkError>) {
    assert!(matches!(
        handshake_response,
        Err(NetworkError::IO(e)) if e.kind() == io::ErrorKind::UnexpectedEof
    ));
}

#[tokio::test]
async fn test_connection() {
    let peer_id = PeerId::new(PublicKey::ED25519(ED25519PublicKey([1u8; 32])));

    let sender_listen_port = 24567;

    let cursor = io::Cursor::new(Vec::new());

    let mut connection =
        Connection::new(cursor, peer_id, sender_listen_port, Duration::from_secs(1));

    // Performe the first test of the handshake request - it should fail with UnexpectedEof error
    // because the stream interal buffer is empty
    assert_unexpected_eof(test_handshake_request(&mut connection).await);

    // Check if the stream internal buffer contains the default handshake request
    {
        seek_to_start(&mut connection).await;
        let peer_message = connection.read_message().await.unwrap();
        assert_end_stream(&mut connection).await;

        let handshake: Handshake = match peer_message.message_type.unwrap() {
            MessageType::Handshake(handshake) => Ok((&handshake).try_into().unwrap()),
            _ => Err(()),
        }
        .unwrap();

        assert_eq!(
            handshake,
            connection.create_handshake(PROTOCOL_VERSION, Default::default(), 0)
        );
    }

    let genesis_id = GenesisId {
        chain_id: "localnet".into(),
        hash: CryptoHash([2u8; 32]),
    };

    // Write HandshakeFailure::GenesisMismatch response to stream internal buffer and test
    // handshake request again
    {
        let handshake_failure = HandshakeFailure::GenesisMismatch(genesis_id.clone());

        connection
            .write_message((&handshake_failure).into())
            .await
            .unwrap();

        assert_unexpected_eof(test_handshake_request(&mut connection).await);
    }

    // Write Handshake response to stream internal buffer and perform the final test
    // of handshake request
    {
        let handshake = connection.create_handshake(PROTOCOL_VERSION, genesis_id, 0);

        connection.write_message((&handshake).into()).await.unwrap();

        assert_eq!(
            test_handshake_request(&mut connection).await.unwrap().0,
            handshake
        );
    }
}
