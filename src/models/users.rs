use std::net::IpAddr;

use actix_web::web;
use bcrypt::hash;
use futures::StreamExt;
use mongodb::{
    bson::{doc, from_document, oid::ObjectId, Document},
    error::Error,
    options::UpdateModifications,
    Collection, Cursor,
};
use serde::{Deserialize, Serialize};

use crate::api::auth::AuthCredentials;

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
    pub profile_img_url: Option<String>,
    #[serde(rename = "activeIps")]
    pub active_ips: Option<Vec<IpAddr>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MinUser {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub email: String,
    pub username: String,
    #[serde(rename = "profileImgUrl")]
    pub profile_img_url: Option<String>,
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
        id: String,
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
                                        "password": 0,
                                        "activeIps": 0
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
                            "activeIps": 0
                        }
                    },
                ],
                None,
            )
            .await;

        Self::parse_aggrigate::<Self>(users).await
    }

    pub async fn get_user_by_query<T>(
        data: web::Data<Tweetbook>,
        query: Document,
    ) -> Result<Vec<T>, Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        let users = Self::get_collection::<T>(data)
            .aggregate(
                vec![
                    query,
                    doc! {
                        "$project": {
                            "followers": 0,
                            "following": 0,
                            "messages": 0,
                        }
                    },
                ],
                None,
            )
            .await;

        Self::parse_aggrigate::<T>(users).await
    }

    pub async fn add_user(
        data: web::Data<Tweetbook>,
        creds: AuthCredentials,
    ) -> Result<MinUser, Error> {
        let cloned_creds = creds.clone();

        let user = Self::get_collection::<Document>(data)
            .insert_one(
                doc! {
                    "username": creds.username.unwrap(),
                    "email": creds.email,
                    "password": hash(creds.password, 10).unwrap(),
                    "activeIps": vec![creds.ip.unwrap().to_string()]
                },
                None,
            )
            .await;

        match user {
            Ok(inserted) => Ok(MinUser {
                id: inserted.inserted_id.as_object_id().unwrap(),
                email: cloned_creds.email,
                username: cloned_creds.password,
                profile_img_url: Some("".to_string()),
            }),
            Err(error) => Err(error),
        }
    }

    pub async fn update_user(
        data: web::Data<Tweetbook>,
        user_id: String,
        update: impl Into<UpdateModifications>,
    ) -> Result<MinUser, Error> {
        let user_updated = Self::get_collection::<MinUser>(data)
            .find_one_and_update(
                doc! {"$expr": {
                    "$eq": ["$_id", {"$toObjectId": user_id}]
                }},
                update,
                None,
            )
            .await;

        match user_updated {
            Ok(user) => Ok(user.unwrap()),
            Err(error) => Err(error),
        }
    }
}
