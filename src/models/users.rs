use actix_web::web;
use futures::StreamExt;
use mongodb::{
    bson::{doc, from_document, oid::ObjectId, Document},
    error::Error,
    Collection, Cursor,
};
use serde::{Deserialize, Serialize};

use super::{init::Tweetbook, messages::Message};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub messages: Option<Vec<Message>>,
    pub email: String,
    pub password: Option<String>,
    pub username: String,
    pub followers: Option<Vec<MinUser>>,
    pub following: Option<Vec<MinUser>>,
    pub bio: Option<String>,
    #[serde(rename = "profileImgUrl")]
    pub profile_img_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MinUser {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub email: String,
    pub username: String,
    pub bio: Option<String>,
    #[serde(rename = "profileImgUrl")]
    pub profile_img_url: String,
}

impl User {
    pub fn get_collection<T>(data: web::Data<Tweetbook>) -> Collection<T> {
        data.db.collection::<T>("users")
    }

    async fn parse_aggrigate<T>(cursor: Result<Cursor<Document>, Error>) -> Result<Vec<T>, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        match cursor {
            Ok(mut users) => {
                let mut result: Vec<T> = vec![];

                while let Some(res) = users.next().await {
                    let usr: T = from_document(res.unwrap()).unwrap();
                    result.push(usr);
                }
                Ok(result)
            }
            Err(error) => Err(error),
        }
    }

    pub async fn get_user_details(
        data: web::Data<Tweetbook>,
        id: &str,
    ) -> Result<Vec<Self>, Error> {
        let users = Self::get_collection::<Self>(data)
            .aggregate(
                vec![
                    doc! {
                        "$match": {
                            "$expr": {
                                "$eq": ["$_id", {"$toObjectId": id}]
                            }
                        }
                    },
                    doc! {
                        "$lookup": {
                            "from": "messages",
                            "localField": "messages",
                            "foreignField": "_id",
                            "pipeline": [
                                {
                                    "$lookup": {
                                        "from": "users",
                                        "localField": "user",
                                        "foreignField": "_id",
                                        "as": "user"
                                    }
                                },
                                {
                                    "$addFields": {
                                        "user": {
                                            "$first": "$user"
                                        }
                                    }
                                },
                                {
                                    "$lookup": {
                                        "from": "users",
                                        "localField": "likes",
                                        "foreignField": "_id",
                                        "as": "likes",
                                    }
                                },
                                {
                                    "$project": {
                                        "followers": 0,
                                        "following": 0,
                                        "messages": 0,
                                        "password": 0
                                    }
                                }
                            ],
                            "as": "messages",
                        }
                    },
                    doc! {
                        "$lookup": {
                            "from": "users",
                            "localField": "followers",
                            "foreignField": "_id",
                            "as": "followers",
                        }
                    },
                    doc! {
                        "$lookup": {
                            "from": "users",
                            "localField": "following",
                            "foreignField": "_id",
                            "as": "following",
                        }
                    },
                    doc! {
                        "$project": {
                            "password": 0,
                        }
                    },
                ],
                None,
            )
            .await;

        Self::parse_aggrigate::<Self>(users).await
    }

    pub async fn get_user_by_email(
        data: web::Data<Tweetbook>,
        email: &str,
    ) -> Result<Vec<Self>, Error> {
        let users = Self::get_collection::<Self>(data)
            .aggregate(
                vec![
                    doc! {
                        "$match": { "email": email }
                    },
                    doc! {
                        "$project": {
                            "followers": 0,
                            "following": 0,
                            "messages": 0
                        }
                    },
                ],
                None,
            )
            .await;

        Self::parse_aggrigate::<Self>(users).await
    }
}
