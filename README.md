# TweetBook

API routes

/api/auth/signin -> POST (login user) DONE

/api/auth/signup -> POST (signup user) DONE

/api/user/profile -> GET (get user's profile), PUT (update user profile) DONE

/api/users/:id/message/:messageId/like -> POST (like a tweet), DELETE (unlike a tweet)

/api/users/:id/follow/:id2 -> POST (follow user with id2), DELETE (unfollow user with id2)

/api/users/:id/:message_id -> GET (get a tweet), DELETE (delete a tweet)

/api/users/tweet -> POST (post a tweet) DONE

/api/tweets -> GET (get all tweets) DONE
