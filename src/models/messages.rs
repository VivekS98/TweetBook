use actix_web::web;
use futures::StreamExt;
use mongodb::{
    bson::{doc, from_document, oid::ObjectId, DateTime},
    Collection,
};
use serde::{Deserialize, Serialize};

use super::{init::Tweetbook, users::MinUser};

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub text: String,
    pub user: Option<MinUser>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime,
    pub likes: Vec<MinUser>,
}

impl Message {
    pub fn get_collection(data: web::Data<Tweetbook>) -> Collection<Self> {
        data.db.collection::<Self>("messages")
    }

    pub async fn get_all_messages(data: web::Data<Tweetbook>) -> Vec<Self> {
        let mut messages = Self::get_collection(data)
            .aggregate(
                vec![
                    doc! {
                        "$lookup": {
                            "from": "users",
                            "localField": "user",
                            "foreignField": "_id",
                            "as": "user",
                        }
                    },
                    doc! {
                        "$addFields": {
                            "user": {
                                "$first": "$user"
                            }
                        }
                    },
                    doc! {
                        "$lookup": {
                            "from": "users",
                            "localField": "likes",
                            "foreignField": "_id",
                            "as": "likes",
                        }
                    },
                    doc! {
                        "$project": {
                            "user.messages": 0,
                            "likes.messages": 0,
                        }
                    },
                ],
                None,
            )
            .await
            .unwrap();

        let mut result: Vec<Self> = vec![];

        while let Some(res) = messages.next().await {
            let msg: Self = from_document(res.unwrap()).unwrap();
            result.push(msg);
        }
        result
    }
}
