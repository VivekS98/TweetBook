# TweetBook

API routes

/api/auth/signin -> POST (login user) DONE

/api/auth/signup -> POST (signup user) DONE

/api/users/:id/ -> GET (get user's tweets), PUT (update user profile) DONE

/api/users/:id/message/:messageId/like -> POST (like a tweet), DELETE (unlike a tweet)

/api/users/:id/follow/:id2 -> POST (follow user with id2), DELETE (unfollow user with id2)

/api/users/:id/notify -> GET (get notifications), PUT (mark notofications)

/api/users/:id/:message_id -> GET (get a tweet), DELETE (delete a tweet)

/api/users/:id/messages -> POST (post a tweet)

/api/messages -> GET (get all tweets) DONE
