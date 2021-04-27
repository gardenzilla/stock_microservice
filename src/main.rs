use gzlib::proto::stock::stock_server::*;
use gzlib::proto::stock::*;
use packman::*;
use prelude::{ServiceError, ServiceResult};
use std::{env, path::PathBuf};
use tokio::sync::{oneshot, Mutex};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

mod prelude;
mod stock;

struct StockService {
  stocks: Mutex<VecPack<stock::Stock>>,
}

impl StockService {
  fn new(db: VecPack<stock::Stock>) -> Self {
    Self {
      stocks: Mutex::new(db),
    }
  }
  // Get next id
  // Iterate over all the IDs and returns the max ID
  // value + 1
  async fn next_id(&self) -> u32 {
    let mut latest_id: u32 = 0;
    self.stocks.lock().await.iter().for_each(|source| {
      let id: u32 = *source.unpack().get_id();
      if id > latest_id {
        latest_id = id;
      }
    });
    latest_id + 1
  }
  async fn create_new(&self, r: CreateNewRequest) -> ServiceResult<StockObject> {
    let new_stock_id = self.next_id().await;
    let new_stock = stock::Stock::new(new_stock_id, r.name, r.description, r.created_by);
    self.stocks.lock().await.insert(new_stock.clone())?;

    // Get the last inserted (this) source
    let new_stock = self
      .stocks
      .lock()
      .await
      .last()
      .ok_or(ServiceError::internal_error("Az új stock nem található!"))?
      .unpack()
      .clone();

    // Return this as SourceObject
    Ok(new_stock.into())
  }
  async fn get_by_id(&self, r: GetByIdRequest) -> ServiceResult<StockObject> {
    // Try find the requested source
    let res = self
      .stocks
      .lock()
      .await
      .find_id(&r.stock_id)
      .map_err(|_| ServiceError::not_found("A megadott raktár nem található"))?
      .unpack()
      .clone();

    // Returns SourceObject
    Ok(res.into())
  }

  async fn update_by_id(&self, r: StockObject) -> ServiceResult<StockObject> {
    // Try find source as mut
    match self.stocks.lock().await.find_id_mut(&r.stock_id) {
      Ok(stock) => {
        // Try update source data
        let res = stock
          .as_mut()
          .unpack()
          .update(r.name, r.description)
          .clone();

        // Return result updated source data
        Ok(res.into())
      }
      Err(_) => Err(ServiceError::not_found("A megadott stock nem található!")),
    }
  }

  async fn get_all(&self) -> ServiceResult<Vec<StockObject>> {
    let res = self
      .stocks
      .lock()
      .await
      .iter()
      .map(|s| s.unpack().clone().into())
      .collect::<Vec<StockObject>>();
    Ok(res)
  }
}

#[tonic::async_trait]
impl gzlib::proto::stock::stock_server::Stock for StockService {
  async fn create_new(
    &self,
    request: Request<CreateNewRequest>,
  ) -> Result<Response<StockObject>, Status> {
    let res = self.create_new(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn update_by_id(
    &self,
    request: Request<StockObject>,
  ) -> Result<Response<StockObject>, Status> {
    let res = self.update_by_id(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn get_by_id(
    &self,
    request: Request<GetByIdRequest>,
  ) -> Result<Response<StockObject>, Status> {
    let res = self.get_by_id(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  type GetAllStream = ReceiverStream<Result<StockObject, Status>>;

  async fn get_all(&self, _request: Request<()>) -> Result<Response<Self::GetAllStream>, Status> {
    // Create channels
    let (mut tx, rx) = tokio::sync::mpsc::channel(4);
    // Get found stock objects
    let res = self.get_all().await?;
    // Send found stock_objects through the channel
    tokio::spawn(async move {
      for ots in res.into_iter() {
        tx.send(Ok(ots)).await.unwrap();
      }
    });
    return Ok(Response::new(ReceiverStream::new(rx)));
  }
}

#[tokio::main]
async fn main() -> prelude::ServiceResult<()> {
  let db: VecPack<stock::Stock> =
    VecPack::load_or_init(PathBuf::from("data/stock")).expect("Error while loading stock db");

  let stock_service = StockService::new(db);

  let addr = env::var("SERVICE_ADDR_STOCK")
    .unwrap_or("[::1]:50073".into())
    .parse()
    .unwrap();

  // Create shutdown channel
  let (tx, rx) = oneshot::channel();

  // Spawn the server into a runtime
  tokio::task::spawn(async move {
    Server::builder()
      .add_service(StockServer::new(stock_service))
      .serve_with_shutdown(addr, async { rx.await.unwrap() })
      .await
  });

  tokio::signal::ctrl_c().await.unwrap();

  println!("SIGINT");

  // Send shutdown signal after SIGINT received
  let _ = tx.send(());

  Ok(())
}
