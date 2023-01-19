use actix_web::web;
use futures::StreamExt;
use mongodb::{
    bson::{doc, from_document, oid::ObjectId, DateTime, Document},
    error::Error,
    Collection, Cursor,
};
use serde::{Deserialize, Serialize};

use crate::models::users::User;

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
    pub fn get_collection<T>(data: web::Data<Tweetbook>) -> Collection<T> {
        data.db.collection::<T>("messages")
    }

    async fn parse_aggrigate<T>(cursor: Result<Cursor<Document>, Error>) -> Result<Vec<T>, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        match cursor {
            Ok(mut message) => {
                let mut result: Vec<T> = vec![];

                while let Some(res) = message.next().await {
                    let msg: T = from_document(res.unwrap()).unwrap();
                    result.push(msg);
                }
                Ok(result)
            }
            Err(error) => Err(error),
        }
    }

    pub async fn get_all_messages(data: web::Data<Tweetbook>) -> Result<Vec<Self>, Error> {
        let messages = Self::get_collection::<Self>(data)
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
            .await;

        Self::parse_aggrigate::<Self>(messages).await
    }

    pub async fn insert_message(
        data: web::Data<Tweetbook>,
        text: &str,
        user_id: String,
    ) -> Result<Message, Error> {
        let message = Self::get_collection::<Document>(data.clone())
            .insert_one(
                doc! {
                    "user": user_id.clone(),
                    "text": text
                },
                None,
            )
            .await;

        match message {
            Ok(inserted) => {
                let message_id = inserted.inserted_id.as_object_id().unwrap();

                let user_resp =
                    User::update_user(data, user_id, doc! { "$push": { "messages": message_id }})
                        .await;

                match user_resp {
                    Ok(user) => Ok(Self {
                        id: message_id,
                        text: text.to_string(),
                        user: Some(user),
                        created_at: DateTime::now(),
                        updated_at: DateTime::now(),
                        likes: vec![],
                    }),
                    Err(error) => {
                        println!("MSG_error {}", error);
                        Err(error)
                    }
                }
            }
            Err(error) => Err(error),
        }
    }
}
