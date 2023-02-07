use std::str::FromStr;

use actix_web::web;
use futures::StreamExt;
use mongodb::{
    bson::{doc, from_document, oid::ObjectId, DateTime, Document},
    error::Error,
    options::UpdateModifications,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct MinMessage {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub text: String,
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

    pub async fn get_message_by_query<T>(
        data: web::Data<Tweetbook>,
        query: Option<Document>,
    ) -> Result<Vec<T>, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let users = Self::get_collection::<T>(data)
            .aggregate(
                vec![
                    query.unwrap_or_else(|| doc! {"$match": {}}),
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

        Self::parse_aggrigate::<T>(users).await
    }

    pub async fn insert_message(
        data: web::Data<Tweetbook>,
        text: String,
        user_id: String,
    ) -> Result<Message, Error> {
        let message = Self::get_collection::<Document>(data.clone())
            .insert_one(
                doc! {
                    "user": ObjectId::from_str(user_id.as_str()).unwrap(),
                    "text": text.clone(),
                    "createdAt": DateTime::now(),
                    "updatedAt": DateTime::now()
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
                        text: text,
                        user: Some(user),
                        created_at: DateTime::now(),
                        updated_at: DateTime::now(),
                        likes: vec![],
                    }),
                    Err(error) => Err(error),
                }
            }
            Err(error) => Err(error),
        }
    }

    pub async fn update_message(
        data: web::Data<Tweetbook>,
        message_id: String,
        update: impl Into<UpdateModifications>,
    ) -> Result<Message, Error> {
        let message_updated = Self::get_collection::<MinMessage>(data.clone())
            .find_one_and_update(
                doc! {"$expr": {
                    "$eq": ["$_id", {"$toObjectId": message_id.clone()}]
                }},
                update,
                None,
            )
            .await;

        match message_updated {
            Ok(message) => {
                let msg_res = Message::get_message_by_query::<Message>(
                    data,
                    Some(doc! {
                        "$match": {
                            "_id": message.unwrap().id
                        }
                    }),
                )
                .await;

                match msg_res {
                    Ok(mut msg) => Ok(msg.remove(0)),
                    Err(error) => Err(error),
                }
            }
            Err(error) => Err(error),
        }
    }

    pub async fn delete_message(
        data: web::Data<Tweetbook>,
        tweet_id: String,
        user_id: String,
    ) -> Result<(), Error> {
        let delete_response = Self::get_collection::<MinMessage>(data.clone())
            .find_one_and_delete(
                doc! {"$and": [
                    {
                        "$expr": {
                            "$eq": ["$_id", {"$toObjectId": tweet_id.clone()}]
                        }
                    },
                    {
                        "$expr": {
                            "$eq": ["$user", {"$toObjectId": user_id.clone()}]
                        }
                    },
                ]},
                None,
            )
            .await;

        match delete_response {
            Ok(deleted_tweet) => {
                let updated_user = User::update_user(
                    data,
                    user_id,
                    doc! { "$pull": { "messages": { "$in": vec![deleted_tweet.unwrap().id]} }},
                )
                .await;

                match updated_user {
                    Ok(_) => Ok(()),
                    Err(error) => Err(error),
                }
            }
            Err(error) => Err(error),
        }
    }
}
