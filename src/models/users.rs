use actix_web::web;
use futures::StreamExt;
use mongodb::{
    bson::{doc, from_document, oid::ObjectId},
    Collection,
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
    pub id: Option<ObjectId>,
    pub email: String,
    pub username: String,
    pub bio: Option<String>,
    #[serde(rename = "profileImgUrl")]
    pub profile_img_url: String,
}

impl User {
    pub fn get_collection(data: web::Data<Tweetbook>) -> Collection<Self> {
        data.db.collection::<Self>("users")
    }

    pub async fn get_users(data: web::Data<Tweetbook>, id: &str) -> Vec<Self> {
        let mut users = Self::get_collection(data)
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
            .await
            .unwrap();

        let mut result: Vec<Self> = vec![];

        while let Some(res) = users.next().await {
            let msg: Self = from_document(res.unwrap()).unwrap();
            result.push(msg);
        }
        result
    }
}
