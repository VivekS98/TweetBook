use actix_web::web;
use futures::StreamExt;
use mongodb::{
    bson::{doc, from_document, oid::ObjectId},
    error::Error,
    Collection,
};
use serde::{Deserialize, Serialize};

use crate::api::auth::Auth;

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
    pub id: Option<ObjectId>,
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

    pub async fn get_user_details(
        data: web::Data<Tweetbook>,
        id: &str,
    ) -> Result<Vec<Self>, Error> {
        let users_response = Self::get_collection::<Self>(data)
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

        match users_response {
            Ok(mut users) => {
                let mut result: Vec<Self> = vec![];

                while let Some(res) = users.next().await {
                    let msg: Self = from_document(res.unwrap()).unwrap();
                    result.push(msg);
                }
                Ok(result)
            }
            Err(error) => Err(error),
        }
    }

    pub async fn get_user(data: web::Data<Tweetbook>, body: web::Json<Auth>) -> Option<User> {
        Self::get_collection(data)
            .find_one(doc! {}, None)
            .await
            .expect("User Not found!")
    }
}
